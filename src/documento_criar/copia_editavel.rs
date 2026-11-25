//! Prepara o par local que `documento salvar` usa, a partir da versão lida.
use crate::copia_documento::ErroCopia;
use crate::documento::{NumeroDocumento, NumeroDocumentoInvalido};
use crate::foto_documento;
use crate::indice_local::IndiceLocal;
use crate::processo_obter::{
    EnvioDoProcesso, ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    obter_foto_atual,
};
use crate::tela_html::{HtmlServido, arvore};
use serde::Serialize;
use std::path::Path;

const ORIENTACAO: &str = "Edite o arquivo e, de preferência, siga os estilos que ele já contém.";

#[derive(Serialize)]
pub(super) struct CopiaEditavel {
    pub(super) arquivo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) orientacao: Option<&'static str>,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroCopiaEditavel {
    Numero(#[from] NumeroDocumentoInvalido),
    Leitura(#[from] ErroDocumentoObter),
    SemVersaoSeguinte,
    Gravar(#[from] ErroCopia),
}

pub(super) fn preparar(
    pasta: &Path,
    numero_sei: &str,
    criacao: &crate::documento_criar::PedidoCriacao<'_>,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    envio: &mut impl EnvioDoProcesso,
) -> Result<CopiaEditavel, ErroCopiaEditavel> {
    let numero = NumeroDocumento::novo(numero_sei)?;
    let pedido = PedidoDoDocumento {
        numero: &numero,
        processo: Some(criacao.nup),
        unidade: criacao.unidade,
        anexo: PedidoDoAnexo::default(),
    };
    let foto = obter_foto_atual(sessao, &pedido, indice, envio)?;
    let edicao = foto
        .versao
        .seguinte()
        .ok_or(ErroCopiaEditavel::SemVersaoSeguinte)?;
    let html = foto.html.as_str();
    // Confira o destino editável antes de gravar a base. Nenhum arquivo
    // existente com bytes diferentes é sobrescrito.
    foto_documento::planejar(pasta, numero.as_str(), edicao, html)?;
    foto_documento::gravar(pasta, numero.as_str(), foto.versao, html)?;
    let arquivo = foto_documento::gravar(pasta, numero.as_str(), edicao, html)?.nome;
    Ok(CopiaEditavel {
        arquivo,
        orientacao: contem_estilos(&foto.html).then_some(ORIENTACAO),
    })
}

pub(crate) fn contem_estilos(html: &HtmlServido) -> bool {
    let dom = arvore(html);
    let mut elementos = dom
        .elementos("html")
        .flat_map(|raiz| std::iter::once(raiz).chain(raiz.descendentes()));
    elementos.any(|elemento| {
        (elemento.nome() == "style" && !elemento.texto().trim().is_empty())
            || elemento
                .atributo("style")
                .is_some_and(|css| !css.trim().is_empty())
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn orienta_por_css_presente_no_html() {
        for html in [
            "<style>.texto { color: blue }</style>",
            "<p STYLE='color:blue'>Texto</p>",
        ] {
            assert!(contem_estilos(&HtmlServido::de_teste(html)));
        }
        for html in [
            "<p class='texto'>Texto</p>",
            "<!-- <style>.texto { color: blue }</style> -->",
            "<style> </style><p style=' '>Texto</p>",
            "<link rel='stylesheet' href='tema.css'>",
        ] {
            assert!(!contem_estilos(&HtmlServido::de_teste(html)));
        }
        let copia = CopiaEditavel {
            arquivo: "0007.v2.html".to_owned(),
            orientacao: None,
        };
        let json = serde_json::to_value(copia).unwrap();
        assert_eq!(json, serde_json::json!({"arquivo": "0007.v2.html"}));
    }
}
