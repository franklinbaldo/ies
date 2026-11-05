//! Inventário das versões locais `DOC.vN.html` (#207), sem sessão, rede
//! ou links de histórico: cada uma é a foto registrada (SHA-256 igual ao
//! que o índice local guardou ao gravá-la) ou um arquivo local não
//! registrado (a edição). Foto registrada e alterada recusa a lista.
use crate::arquivo_copia::ler_regular;
use crate::copia_documento::{self, ErroCopia, Registro};
use crate::documento::NumeroDocumento;
use crate::foto_documento::{nome, sha256};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErroListagem {
    LimiteDeCopias,
    CopiaInvalida(ErroCopia),
}

impl ErroListagem {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::LimiteDeCopias => "limite_copias_locais",
            Self::CopiaInvalida(erro) => erro.subcodigo(),
        }
    }
}

/// Uma versão local, sem o conteúdo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VersaoLocal {
    pub(crate) documento: String,
    pub(crate) versao: u32,
    pub(crate) bytes: usize,
    /// `true`: a foto que o SEI serviu, conferida pelo SHA-256 registrado.
    pub(crate) foto: bool,
}

/// Consulta somente os arquivos do número fornecido. Um arquivo inseguro
/// recusa a lista inteira, não é omitido.
pub(crate) fn listar(
    pasta: &Path,
    documento: &NumeroDocumento,
    registro: Registro<'_>,
) -> Result<Vec<VersaoLocal>, ErroListagem> {
    let versoes =
        copia_documento::versoes_locais(pasta, documento).map_err(ErroListagem::CopiaInvalida)?;
    if versoes.len() > 128 {
        return Err(ErroListagem::LimiteDeCopias);
    }
    let mut lista = Vec::with_capacity(versoes.len());
    for versao in versoes {
        let alvo = pasta.join(nome(documento.as_str(), versao));
        let bytes =
            ler_regular(&alvo, ErroCopia::LeituraFalhou).map_err(ErroListagem::CopiaInvalida)?;
        let foto = match registro(versao) {
            None => false,
            Some(sha) if sha == sha256(&bytes) => true,
            Some(_) => return Err(ErroListagem::CopiaInvalida(ErroCopia::FotoDivergente)),
        };
        lista.push(VersaoLocal {
            documento: documento.as_str().to_owned(),
            versao: versao.numero(),
            bytes: bytes.len(),
            foto,
        });
    }
    Ok(lista)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::documento_versoes::VersaoDocumento;
    use crate::foto_documento::{gravar, tests::pasta};
    use std::collections::BTreeMap;
    use std::fs;

    const CORPO: &str = "<p>corpo de teste</p>";

    fn listar_com(
        pasta: &Path,
        doc: &str,
        registro: &BTreeMap<u32, String>,
    ) -> Result<Vec<VersaoLocal>, ErroListagem> {
        let consulta = |v: VersaoDocumento| registro.get(&v.numero()).cloned();
        listar(pasta, &NumeroDocumento::novo(doc).unwrap(), &consulta)
    }

    fn foto(pasta: &Path, numero: &str, n: u32) -> String {
        gravar(pasta, numero, VersaoDocumento::nova(n).unwrap(), CORPO)
            .unwrap()
            .sha256
    }

    #[test]
    fn lista_fotos_e_edicoes_em_ordem_numerica_sem_expor_texto() {
        let pasta = pasta();
        let registro =
            BTreeMap::from([(2, foto(&pasta, "0007", 2)), (10, foto(&pasta, "0007", 10))]);
        foto(&pasta, "0007", 3);
        let lista = listar_com(&pasta, "0007", &registro).unwrap();
        assert_eq!(
            lista.iter().map(|c| (c.versao, c.foto)).collect::<Vec<_>>(),
            [(2, true), (3, false), (10, true)]
        );
        let serializado = serde_json::to_string(&lista).unwrap();
        assert!(!serializado.contains("corpo") && !serializado.contains("<p>"));
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn foto_registrada_alterada_recusa_a_lista() {
        let pasta = pasta();
        foto(&pasta, "0007", 2);
        let registro = BTreeMap::from([(2, "0".repeat(64))]);
        assert_eq!(
            listar_com(&pasta, "0007", &registro),
            Err(ErroListagem::CopiaInvalida(ErroCopia::FotoDivergente))
        );
        fs::remove_dir_all(pasta).unwrap();
    }

    #[test]
    fn duas_grafias_outro_numero_e_limite() {
        let pasta = pasta();
        foto(&pasta, "0008", 1);
        assert_eq!(listar_com(&pasta, "0007", &BTreeMap::new()), Ok(Vec::new()));
        fs::write(pasta.join("0007.v2.html"), "um").unwrap();
        fs::write(pasta.join("0007.v02.html"), "outro").unwrap();
        assert_eq!(
            listar_com(&pasta, "0007", &BTreeMap::new()),
            Err(ErroListagem::CopiaInvalida(ErroCopia::VersaoDuplicada))
        );
        fs::remove_dir_all(&pasta).unwrap();
        let pasta = crate::foto_documento::tests::pasta();
        for n in 1..=129 {
            fs::write(pasta.join(format!("0007.v{n}.html")), "teste").unwrap();
        }
        assert_eq!(
            listar_com(&pasta, "0007", &BTreeMap::new()),
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
            listar_com(&link, "0007", &BTreeMap::new()),
            Err(ErroListagem::CopiaInvalida(ErroCopia::DestinoInseguro))
        );
        fs::remove_file(link).unwrap();
        fs::remove_dir_all(pasta).unwrap();
    }
}
