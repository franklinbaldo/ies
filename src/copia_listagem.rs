//! Inventário de cópias de trabalho locais, sem sessão, rede ou links de histórico.
use crate::copia_documento::{ErroCopia, PreviaCopia};
use crate::documento::NumeroDocumento;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroListagem {
    NaoImplementado,
    DiretorioInseguro,
    LeituraDoDiretorioFalhou,
    NomeDaVersaoInvalido,
    VersaoDuplicada,
    LimiteDeCopias,
    CopiaInvalida(ErroCopia),
}

impl ErroListagem {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::NaoImplementado => "listagem_copias_nao_implementada",
            Self::DiretorioInseguro => "diretorio_copias_inseguro",
            Self::LeituraDoDiretorioFalhou => "leitura_diretorio_copias_falhou",
            Self::NomeDaVersaoInvalido => "nome_versao_local_invalido",
            Self::VersaoDuplicada => "versao_local_repetida",
            Self::LimiteDeCopias => "limite_copias_locais",
            Self::CopiaInvalida(erro) => erro.subcodigo(),
        }
    }
}

/// Consulta somente as cópias de trabalho para o número fornecido.
/// Uma cópia insegura deve recusar a lista inteira, não ser omitida.
pub(crate) fn listar(
    _pasta: &Path,
    _documento: &NumeroDocumento,
) -> Result<Vec<PreviaCopia>, ErroListagem> {
    Err(ErroListagem::NaoImplementado)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::copia_documento::{self, FonteHtml, IdentidadeCopia};
    use crate::documento_versoes::VersaoDocumento;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pasta() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ies-copias-listagem-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    fn criar(pasta: &Path, numero: &str, versao: u32) -> copia_documento::CaminhosCopia {
        let id = IdentidadeCopia {
            documento: NumeroDocumento::novo(numero).unwrap(),
            id_interno: "42".to_owned(),
            instalacao: "https://sei.exemplo/sei".to_owned(),
            versao: VersaoDocumento::nova(versao).unwrap(),
        };
        copia_documento::preparar(
            pasta,
            &id,
            FonteHtml::EditorVerificado {
                html: "<p>corpo de teste</p>",
                assinado: false,
            },
        )
        .unwrap()
    }

    #[test]
    fn lista_versoes_validas_ordenadas_e_alteracoes_sem_expor_texto() {
        let pasta = pasta();
        let doc = NumeroDocumento::novo("0007").unwrap();
        criar(&pasta, "0007", 10);
        let paths = criar(&pasta, "0007", 2);
        fs::write(&paths.trabalho, "<p>rascunho alterado sigiloso</p>").unwrap();
        let copias = listar(&pasta, &doc).unwrap();
        assert_eq!(copias.iter().map(|c| c.versao_base).collect::<Vec<_>>(), [2, 10]);
        assert!(copias[0].alterado);
        assert!(!copias[1].alterado);
        let serializado = serde_json::to_string(&copias).unwrap();
        assert!(!serializado.contains("sigiloso"));
        assert!(!serializado.contains("<p>"));
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn copia_sem_base_e_manifesto_nao_pode_sumir_da_listagem() {
        let pasta = pasta();
        let doc = NumeroDocumento::novo("0007").unwrap();
        fs::write(pasta.join("0007.v2.html"), "corpo solto").unwrap();
        assert_eq!(
            listar(&pasta, &doc),
            Err(ErroListagem::CopiaInvalida(ErroCopia::BaseAusente))
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn duas_grafias_para_a_mesma_versao_sao_ambiguas() {
        let pasta = pasta();
        let doc = NumeroDocumento::novo("0007").unwrap();
        fs::write(pasta.join("0007.v2.html"), "um").unwrap();
        fs::write(pasta.join("0007.v02.html"), "outro").unwrap();
        assert_eq!(listar(&pasta, &doc), Err(ErroListagem::VersaoDuplicada));
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn sem_copias_do_numero_retorna_lista_vazia_sem_afirmar_ausencia_remota() {
        let pasta = pasta();
        criar(&pasta, "0008", 1);
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(listar(&pasta, &doc), Ok(Vec::new()));
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn diretorio_simbolico_nao_eh_percorrido() {
        use std::os::unix::fs::symlink;
        let pasta = pasta();
        let link = pasta.with_extension("atalho");
        symlink(&pasta, &link).unwrap();
        let doc = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(listar(&link, &doc), Err(ErroListagem::DiretorioInseguro));
        fs::remove_file(link).unwrap();
        fs::remove_dir_all(pasta).unwrap();
    }
}
