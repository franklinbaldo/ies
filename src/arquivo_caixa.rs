//! Exportação local de caixa verificada, nunca sobrepõe arquivo preexistente.
//! Ancoragem em descritores Linux impede desvio de JSON por troca de diretório.
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErroArquivoCaixa {
    DestinoSemNomeDeArquivo,
    DiretorioDeDestinoAusente,
    DiretorioLinkSimbolico,
    DestinoJaExiste,
    EscritaDeArquivoFalhou,
}
impl ErroArquivoCaixa {
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino_da_caixa_sem_nome_de_arquivo",
            Self::DiretorioDeDestinoAusente => "diretorio_do_arquivo_da_caixa_ausente",
            Self::DiretorioLinkSimbolico => "diretorio_do_arquivo_da_caixa_link_simbolico",
            Self::DestinoJaExiste => "arquivo_da_caixa_ja_existe",
            Self::EscritaDeArquivoFalhou => "escrita_de_arquivo_da_caixa_falhou",
        }
    }
    pub fn mensagem(self) -> &'static str {
        match self {
            Self::DestinoSemNomeDeArquivo => "destino da caixa não tem nome de arquivo",
            Self::DiretorioDeDestinoAusente => "diretório de destino não existe",
            Self::DiretorioLinkSimbolico => "diretório de destino atravessa link simbólico",
            Self::DestinoJaExiste => "arquivo de destino já existe e não será substituído",
            Self::EscritaDeArquivoFalhou => "não foi possível gravar a caixa em arquivo privado",
        }
    }
}

/// Pré-validação sem rede: evita GET administrativo quando o destino é inviável.
/// A exclusividade final depende de create_new, não desta inspeção.
pub fn conferir_destino(destino: &Path) -> Result<(), ErroArquivoCaixa> {
    use std::io::ErrorKind;
    if destino.file_name().is_none() {
        return Err(ErroArquivoCaixa::DestinoSemNomeDeArquivo);
    }
    let pai = destino
        .parent()
        .ok_or(ErroArquivoCaixa::DiretorioDeDestinoAusente)?;
    for ancestral in pai.ancestors() {
        if ancestral.as_os_str().is_empty() {
            continue;
        }
        let meta = std::fs::symlink_metadata(ancestral)
            .map_err(|_| ErroArquivoCaixa::DiretorioDeDestinoAusente)?;
        if meta.file_type().is_symlink() {
            return Err(ErroArquivoCaixa::DiretorioLinkSimbolico);
        }
        if !meta.is_dir() {
            return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente);
        }
    }
    match std::fs::symlink_metadata(destino) {
        Ok(_) => Err(ErroArquivoCaixa::DestinoJaExiste),
        Err(erro) if erro.kind() == ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ErroArquivoCaixa::EscritaDeArquivoFalhou),
    }
}

#[cfg(target_os = "linux")]
fn diretorio_ancorado(pai: &Path) -> Result<std::fs::File, ErroArquivoCaixa> {
    use std::fs::{File, OpenOptions};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    use std::path::{Component, PathBuf};

    let mut atual = File::open(if pai.is_absolute() { "/" } else { "." })
        .map_err(|_| ErroArquivoCaixa::DiretorioDeDestinoAusente)?;
    for componente in pai.components() {
        let nome = match componente {
            Component::RootDir | Component::CurDir => continue,
            Component::ParentDir => std::ffi::OsStr::new(".."),
            Component::Normal(nome) => nome,
            Component::Prefix(_) => return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente),
        };
        let proximo = PathBuf::from(format!("/proc/self/fd/{}", atual.as_raw_fd())).join(nome);
        let mut opcoes = OpenOptions::new();
        // Linux: O_DIRECTORY | O_NOFOLLOW. Ausência de /proc falha fechada.
        opcoes.read(true).custom_flags(0o600_000);
        let aberto = opcoes.open(&proximo).map_err(|erro| {
            if erro.raw_os_error() == Some(40) {
                ErroArquivoCaixa::DiretorioLinkSimbolico
            } else {
                ErroArquivoCaixa::DiretorioDeDestinoAusente
            }
        })?;
        if !aberto
            .metadata()
            .map_err(|_| ErroArquivoCaixa::DiretorioDeDestinoAusente)?
            .is_dir()
        {
            return Err(ErroArquivoCaixa::DiretorioDeDestinoAusente);
        }
        atual = aberto;
    }
    Ok(atual)
}

fn gravar_com_intervalo(
    destino: &Path,
    valor: &Value,
    entre_ancoragem_e_abertura: impl FnOnce(),
) -> Result<usize, ErroArquivoCaixa> {
    use std::fs::OpenOptions;
    use std::io::{ErrorKind, Write};
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    conferir_destino(destino)?;
    #[cfg(target_os = "linux")]
    let pai = diretorio_ancorado(
        destino
            .parent()
            .ok_or(ErroArquivoCaixa::DiretorioDeDestinoAusente)?,
    )?;
    let bytes =
        serde_json::to_vec_pretty(valor).map_err(|_| ErroArquivoCaixa::EscritaDeArquivoFalhou)?;
    let mut opcoes = OpenOptions::new();
    opcoes.write(true).create_new(true);
    #[cfg(unix)]
    opcoes.mode(0o600);
    entre_ancoragem_e_abertura();
    #[cfg(target_os = "linux")]
    let caminho = {
        use std::os::fd::AsRawFd;
        std::path::PathBuf::from(format!("/proc/self/fd/{}", pai.as_raw_fd()))
            .join(destino.file_name().ok_or(ErroArquivoCaixa::DestinoSemNomeDeArquivo)?)
    };
    #[cfg(not(target_os = "linux"))]
    let caminho = destino.to_path_buf();
    let mut arquivo = opcoes.open(&caminho).map_err(|erro| {
        if erro.kind() == ErrorKind::AlreadyExists {
            ErroArquivoCaixa::DestinoJaExiste
        } else {
            ErroArquivoCaixa::EscritaDeArquivoFalhou
        }
    })?;
    if arquivo
        .write_all(&bytes)
        .and_then(|()| arquivo.sync_all())
        .is_err()
    {
        // Não remover por pathname: um concorrente pode ter trocado ancestrais.
        // Um parcial permanece privado (0600) para recuperação supervisionada.
        return Err(ErroArquivoCaixa::EscritaDeArquivoFalhou);
    }
    Ok(bytes.len())
}

/// Grava JSON em arquivo novo e 0600; Linux ancora o pai contra symlink concorrente.
pub fn gravar(destino: &Path, valor: &Value) -> Result<usize, ErroArquivoCaixa> {
    gravar_com_intervalo(destino, valor, || {})
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn pasta() -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "ies-exporta-caixa-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        p
    }
    #[test]
    fn exporta_json_legivel_sem_sobrescrever() {
        let dir = pasta();
        let nome = dir.join("caixa.json");
        let dado =
            serde_json::json!({"unidade":"ABC","recebidos":{"total_declarado":0,"processos":[]}});
        let tamanho = gravar(&nome, &dado).unwrap();
        assert!(tamanho > 10);
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&nome).unwrap()).unwrap(),
            dado
        );
        assert_eq!(gravar(&nome, &dado), Err(ErroArquivoCaixa::DestinoJaExiste));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&nome).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn nao_cria_diretorio_implicitamente() {
        let dir = pasta();
        assert_eq!(
            gravar(&dir.join("ausente/caixa.json"), &serde_json::json!({})),
            Err(ErroArquivoCaixa::DiretorioDeDestinoAusente)
        );
        fs::remove_dir_all(dir).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn nao_segue_link_simbolico_no_destino() {
        use std::os::unix::fs::symlink;
        let dir = pasta();
        let alvo = dir.join("original.txt");
        fs::write(&alvo, "NAO ALTERAR").unwrap();
        let link = dir.join("caixa.json");
        symlink(&alvo, &link).unwrap();
        assert_eq!(
            gravar(&link, &serde_json::json!({})),
            Err(ErroArquivoCaixa::DestinoJaExiste)
        );
        assert_eq!(fs::read_to_string(&alvo).unwrap(), "NAO ALTERAR");
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn troca_de_ancestral_nao_redireciona_json_confidencial() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let saida = raiz.join("saida");
        let antiga = raiz.join("saida-original");
        let invasor = raiz.join("invasor");
        fs::create_dir(&saida).unwrap();
        fs::create_dir(&invasor).unwrap();
        let destino = saida.join("caixa.json");
        let esperado = serde_json::json!({"processos":["PRIVADO"]});
        let resultado = gravar_com_intervalo(&destino, &esperado, || {
            fs::rename(&saida, &antiga).unwrap();
            symlink(&invasor, &saida).unwrap();
        });
        let texto = fs::read_to_string(antiga.join("caixa.json")).unwrap();
        assert!(resultado.is_ok());
        assert_eq!(serde_json::from_str::<Value>(&texto).unwrap(), esperado);
        assert!(!invasor.join("caixa.json").exists());
        fs::remove_dir_all(raiz).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn arquivo_simbolico_pendente_e_ancestral_simbolico_falham_na_previa() {
        use std::os::unix::fs::symlink;
        let raiz = pasta();
        let destino = raiz.join("caixa.json");
        symlink(raiz.join("ausente"), &destino).unwrap();
        assert_eq!(conferir_destino(&destino), Err(ErroArquivoCaixa::DestinoJaExiste));
        fs::remove_file(&destino).unwrap();
        let real = raiz.join("real");
        fs::create_dir(&real).unwrap();
        let alias = raiz.join("alias");
        symlink(&real, &alias).unwrap();
        assert_eq!(
            conferir_destino(&alias.join("caixa.json")),
            Err(ErroArquivoCaixa::DiretorioLinkSimbolico)
        );
        fs::remove_dir_all(raiz).unwrap();
    }

}
