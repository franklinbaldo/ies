//! Gravação atômica da sessão (temporário `0600`, `fsync`, `rename`), o
//! marcador de geração e o lock entre processos, só com a `std`.
//! `<url-controle>.lock` e `<url-controle>.geracao` ficam ao lado do
//! arquivo da URL do Controle.
use super::ErroLogin;
use crate::configuracao::ArquivosDeAcesso;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Lock exclusivo do login; liberado quando sai de escopo.
#[derive(Debug)]
pub(crate) struct TravaDoLogin {
    _arquivo: File,
}

impl TravaDoLogin {
    /// Espera até `espera` pelo lock de `<url-controle>.lock`.
    pub(crate) fn obter(url_controle: &Path, espera: Duration) -> Result<Self, ErroLogin> {
        let _ = (url_controle, espera);
        todo!("CONTRACT: lock do login")
    }
}

/// `<arquivo><sufixo>` no mesmo diretório.
pub(crate) fn caminho_ao_lado(arquivo: &Path, sufixo: &str) -> PathBuf {
    let mut nome = arquivo.as_os_str().to_owned();
    nome.push(sufixo);
    PathBuf::from(nome)
}

/// `false` se o marcador existe e não corresponde a estes conteúdos;
/// `true` sem marcador (sessão preparada fora do `ies login`).
pub(crate) fn mesma_geracao(arquivos: &ArquivosDeAcesso, url: &str, cookie: &str) -> bool {
    let _ = (arquivos, url, cookie);
    todo!("CONTRACT: geração")
}

/// Grava cookie, URL e, por último, o marcador da nova geração.
pub(crate) fn gravar(
    arquivos: &ArquivosDeAcesso,
    instalacao: &str,
    url: &str,
    cookie: &str,
) -> Result<u64, ErroLogin> {
    let _ = (arquivos, instalacao, url, cookie);
    todo!("CONTRACT: gravar sessão")
}
