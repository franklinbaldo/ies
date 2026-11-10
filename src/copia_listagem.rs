//! Inventário das versões locais `DOC.vN.html` (#207), sem sessão, rede,
//! índice ou links de histórico: só os arquivos do diretório, sem dizer
//! qual é foto e qual é edição, nem conferir nenhum com o SEI.
use crate::arquivo_copia::ler_regular;
use crate::copia_documento::{self, ErroCopia};
use crate::documento::NumeroDocumento;
use crate::foto_documento::nome;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroListagem {
    LimiteDeCopias,
    #[causa(interna)]
    CopiaInvalida(ErroCopia),
}

/// Uma versão local, sem o conteúdo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VersaoLocal {
    pub(crate) documento: String,
    pub(crate) versao: u32,
    pub(crate) bytes: usize,
}

/// Consulta somente os arquivos do número fornecido. Um arquivo inseguro
/// recusa a lista inteira, não é omitido.
pub(crate) fn listar(
    pasta: &Path,
    documento: &NumeroDocumento,
) -> Result<Vec<VersaoLocal>, ErroListagem> {
    let versoes = copia_documento::versoes_locais(pasta, documento)?;
    if versoes.len() > 128 {
        return Err(ErroListagem::LimiteDeCopias);
    }
    let mut lista = Vec::with_capacity(versoes.len());
    for versao in versoes {
        let alvo = pasta.join(nome(documento.as_str(), versao));
        let bytes = ler_regular(&alvo, ErroCopia::LeituraFalhou)?;
        lista.push(VersaoLocal {
            documento: documento.as_str().to_owned(),
            versao: versao.numero(),
            bytes: bytes.len(),
        });
    }
    Ok(lista)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::documento_versoes::VersaoDocumento;
    use crate::foto_documento::{gravar, tests::pasta};
    use std::fs;

    const CORPO: &str = "<p>corpo de teste</p>";

    fn listar_com(pasta: &Path, doc: &str) -> Result<Vec<VersaoLocal>, ErroListagem> {
        listar(pasta, &NumeroDocumento::novo(doc).unwrap())
    }

    fn foto(pasta: &Path, numero: &str, n: u32) {
        gravar(pasta, numero, VersaoDocumento::nova(n).unwrap(), CORPO).unwrap();
    }

    #[test]
    fn lista_versoes_locais_em_ordem_numerica_sem_expor_texto_nem_foto() {
        let pasta = pasta();
        foto(&pasta, "0007", 2);
        foto(&pasta, "0007", 10);
        fs::write(pasta.join("0007.v3.html"), "editada").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let modo = fs::Permissions::from_mode(0o600);
            fs::set_permissions(pasta.join("0007.v3.html"), modo).unwrap();
        }
        let lista = listar_com(&pasta, "0007").unwrap();
        assert_eq!(
            lista
                .iter()
                .map(|c| (c.versao, c.bytes))
                .collect::<Vec<_>>(),
            [(2, CORPO.len()), (3, 7), (10, CORPO.len())]
        );
        let serializado = serde_json::to_string(&lista).unwrap();
        assert!(!serializado.contains("corpo") && !serializado.contains("<p>"));
        assert!(!serializado.contains("foto"), "{serializado}");
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn duas_grafias_outro_numero_e_limite() {
        let pasta = pasta();
        foto(&pasta, "0008", 1);
        assert_eq!(listar_com(&pasta, "0007"), Ok(Vec::new()));
        fs::write(pasta.join("0007.v2.html"), "um").unwrap();
        fs::write(pasta.join("0007.v02.html"), "outro").unwrap();
        assert_eq!(
            listar_com(&pasta, "0007"),
            Err(ErroListagem::CopiaInvalida(ErroCopia::Html(
                crate::erro_html::ErroHtml::repetido("versao_local_repetida")
            )))
        );
        fs::remove_dir_all(&pasta).unwrap();
        let pasta = crate::foto_documento::tests::pasta();
        for n in 1..=129 {
            fs::write(pasta.join(format!("0007.v{n}.html")), "teste").unwrap();
        }
        assert_eq!(
            listar_com(&pasta, "0007"),
            Err(ErroListagem::LimiteDeCopias)
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn diretorio_simbolico_nao_eh_percorrido() {
        use std::os::unix::fs::symlink;
        let pasta = pasta();
        let link = pasta.with_extension("atalho");
        symlink(&pasta, &link).unwrap();
        assert_eq!(
            listar_com(&link, "0007"),
            Err(ErroListagem::CopiaInvalida(ErroCopia::DestinoInseguro))
        );
        fs::remove_file(link).unwrap();
        fs::remove_dir_all(pasta).unwrap();
    }
}
