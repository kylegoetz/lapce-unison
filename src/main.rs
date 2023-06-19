use std::{
  fs::{self, /*File*/},
  io::{/*self,*/ Read, Write},
  path::PathBuf,
};

use anyhow::{anyhow, Result};
use lapce_plugin::{
  psp_types::{
    lsp_types::{
      request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, MessageType, Url,
    },
    Request,
  },
  register_plugin, /*Http,*/ LapcePlugin, VoltEnvironment, PLUGIN_RPC,
};
use serde_json::Value;
// use zip::ZipArchive;

#[derive(Default)]
struct State {}

register_plugin!(State);

macro_rules! string {
  ( $x:expr ) => {
    String::from($x)
  };
}

macro_rules! ok {
  ( $x:expr ) => {
    match ($x) {
      | Ok(v) => v,
      | Err(e) => return Err(anyhow!(e)),
    }
  };
}

const UCM_VERSION: &str = "release/M4i";

fn initialize(params: InitializeParams) -> Result<()> {
  let document_selector: DocumentSelector = vec![
    DocumentFilter {
      language: Some(string!("Unison")),
      pattern: Some(string!("**/*.{u}")),
      scheme: None,
    },
  ];
  let mut ucm_version = string!(UCM_VERSION);
  // let mut clangd_version = string!(CLANGD_VERSION);
  let mut server_args = vec![];

  if let Some(options) = params.initialization_options.as_ref() {
    if let Some(volt) = options.get("volt") {
      if let Some(args) = volt.get("serverArgs") {
        if let Some(args) = args.as_array() {
          for arg in args {
            if let Some(arg) = arg.as_str() {
              server_args.push(string!(arg));
            }
          }
        }
      }
      if let Some(server_path) = volt.get("serverPath") {
        if let Some(server_path) = server_path.as_str() {
          if !server_path.is_empty() {
            let server_uri = ok!(Url::parse(&format!("urn:{}", server_path)));
            PLUGIN_RPC.start_lsp(
              server_uri,
              server_args,
              document_selector,
              params.initialization_options,
            );
            return Ok(());
          }
        }
      }
      if let Some(ucm_version_val) = options.get("ucmVersion") {
        if let Some(ucm_version_str) = ucm_version_val.as_str() {
          let trimmed_ucm_version = ucm_version_str.trim();
          if !trimmed_ucm_version.is_empty() {
            ucm_version = string!(trimmed_ucm_version);
          }
        }
      }
      // if let Some(ucm_version) = options.get("ucmVersion") {
      //   if let Some(ucm_version) = ucm_version.as_str() {
      //     let ucm_version = ucm_version.trim();
      //     if !ucm_version.is_empty() {
      //       ucm_version = string!(ucm_version)
      //     }
      //   }
      // }
    }
  }

  PLUGIN_RPC.stderr(&format!("ucm: {ucm_version}"));

  let _ = match VoltEnvironment::architecture().as_deref() {
    | Ok("x86_64") => "x86_64",
    | Ok(v) => return Err(anyhow!("Unsupported ARCH: {}", v)),
    | Err(e) => return Err(anyhow!("Error ARCH: {}", e)),
  };

  let mut last_ver = ok!(fs::OpenOptions::new()
    .create(true)
    .write(true)
    .read(true)
    .open(".ucm_ver"));
  let mut buf = String::new();
  ok!(last_ver.read_to_string(&mut buf));

  let mut server_path = PathBuf::from(format!("ucm_{ucm_version}"));
  server_path = server_path.join("bin");

  // if buf.trim().is_empty() || buf.trim() != clangd_version {
  //   if buf.trim() != clangd_version {
  //   ok!(fs::remove_dir_all(&server_path));
  // }

  // let zip_file = match VoltEnvironment::operating_system().as_deref() {
  //   | Ok("macos") => PathBuf::from(format!("clangd-mac-{ucm_version}.zip")),
  //   | Ok("linux") => PathBuf::from(format!("clangd-linux-{ucm_version}.zip")),
  //   | Ok("windows") => PathBuf::from(format!("clangd-windows-{ucm_version}.zip")),
  //   | Ok(v) => return Err(anyhow!("Unsupported OS: {}", v)),
  //   | Err(e) => return Err(anyhow!("Error OS: {}", e)),
  // };

  // let download_url = format!(
  //   "https://github.com/clangd/clangd/releases/download/{clangd_version}/{}",
  //   zip_file.display()
  // );

  // let mut resp = ok!(Http::get(&download_url));
  // PLUGIN_RPC.stderr(&format!("STATUS_CODE: {:?}", resp.status_code));
  // let body = ok!(resp.body_read_all());
  // ok!(fs::write(&zip_file, body));

  // let mut zip = ok!(ZipArchive::new(ok!(File::open(&zip_file))));

  // for i in 0..zip.len() {
  //   let mut file = ok!(zip.by_index(i));
  //   let outpath = match file.enclosed_name() {
  //     | Some(path) => path.to_owned(),
  //     | None => continue,
  //   };

  //   if (*file.name()).ends_with('/') {
  //     ok!(fs::create_dir_all(&outpath));
  //   } else {
  //     if let Some(p) = outpath.parent() {
  //       if !p.exists() {
  //         ok!(fs::create_dir_all(&p));
  //       }
  //     }
  //     let mut outfile = ok!(File::create(&outpath));
  //     ok!(io::copy(&mut file, &mut outfile));
  //   }

  //   ok!(fs::remove_file(&zip_file));
  // }
  // }

  ok!(last_ver.write_all(ucm_version.as_bytes()));

  match VoltEnvironment::operating_system().as_deref() {
    | Ok("windows") => {
      server_path = server_path.join("ucm.exe");
    }
    | _ => {
      // server_path = server_path.join("ucm");
      server_path = PathBuf::from("/opt/homebrew/bin/ucm".to_string());
    }
  };

  let volt_uri = ok!(VoltEnvironment::uri());
  let server_path = match server_path.to_str() {
    | Some(v) => v,
    | None => return Err(anyhow!("server_path.to_str() failed")),
  };
  let server_uri = ok!(ok!(Url::parse(&volt_uri)).join(server_path));

  PLUGIN_RPC.start_lsp(
    server_uri,
    server_args,
    document_selector,
    params.initialization_options,
  );

  Ok(())
}

impl LapcePlugin for State {
  fn handle_request(&mut self, _id: u64, method: String, params: Value) {
    #[allow(clippy::single_match)]
    match method.as_str() {
      | Initialize::METHOD => {
        let params: InitializeParams = serde_json::from_value(params).unwrap();
        if let Err(e) = initialize(params) {
          PLUGIN_RPC.window_log_message(MessageType::ERROR, e.to_string());
          PLUGIN_RPC.window_show_message(MessageType::ERROR, e.to_string());
        };
      }
      | _ => {}
    }
  }
}
