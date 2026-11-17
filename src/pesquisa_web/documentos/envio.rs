//! O envio da pesquisa em documentos: o 302 do número SEI exato (com
//! `id_documento`, conferido como o de `processo buscar` e nunca seguido)
//! e o 200 classificado pela mesma barra, com as linhas de documento.
use super::{ListaDeDocumentos, RespostaDeDocumentos, ler_documentos};
use crate::formulario_web::RespostaDoEnvio;
use crate::instalacao::Instalacao;
use crate::pesquisa_web::alvo::AlvoDaPesquisa;
use crate::pesquisa_web::formulario_pesquisa::{EnvioInterpretado, ErroPesquisaProtocolo};
use crate::pesquisa_web::paginacao::Contagem;
use crate::pesquisa_web::redirecionamento::{documento_do_trabalho, interpretar_redirecionamento};
use crate::pesquisa_web::regras_formulario::PesquisarEm;
use crate::pesquisa_web::resposta::{
    Classificada, ErroRespostaDaPesquisa, FaixaDaPesquisa, classificar_resposta,
};

/// A resposta 200 da pesquisa em documentos (`na_pagina`: a faixa esperada
/// e o número da página pedida por `--pagina N`).
pub(crate) fn ler_resposta_de_documentos(
    html: &crate::tela_html::HtmlServido,
    na_pagina: Option<(FaixaDaPesquisa, u64)>,
) -> Result<RespostaDeDocumentos, ErroRespostaDaPesquisa> {
    let ler = |html: &crate::tela_html::HtmlServido| {
        ler_documentos(html).map_err(ErroRespostaDaPesquisa::Documento)
    };
    Ok(match classificar_resposta(html, na_pagina, ler)? {
        Classificada::SemResultado => RespostaDeDocumentos::SemResultado,
        Classificada::Lista {
            itens,
            total_declarado,
            faixa,
            linhas_ausentes_da_faixa,
        } => RespostaDeDocumentos::Lista(ListaDeDocumentos {
            documentos: itens,
            total_declarado,
            faixa,
            linhas_ausentes_da_faixa,
        }),
    })
}

/// `ies documento buscar`: `#optDocumentos`, linhas de documento.
pub(crate) struct EmDocumentos;

impl AlvoDaPesquisa for EmDocumentos {
    type Resposta = RespostaDeDocumentos;
    const EM: PesquisarEm = PesquisarEm::Documentos;
    fn interpretar(
        enviada: RespostaDoEnvio,
        instalacao: &Instalacao,
    ) -> Result<EnvioInterpretado<RespostaDeDocumentos>, ErroPesquisaProtocolo> {
        match enviada {
            RespostaDoEnvio::Html(html) => ler_resposta_de_documentos(&html, None)
                .map(|resposta| EnvioInterpretado {
                    resposta,
                    html: Some(html),
                    trabalho: None,
                })
                .map_err(ErroPesquisaProtocolo::Resposta),
            RespostaDoEnvio::Redirecionamento(redirecionamento) => {
                let p = interpretar_redirecionamento(redirecionamento, instalacao)?;
                let documento = documento_do_trabalho(&p.trabalho)?;
                // A URL de trabalho é descartada: abrir a árvore não é buscar.
                Ok(EnvioInterpretado {
                    resposta: RespostaDeDocumentos::DocumentoUnico {
                        documento,
                        processo: p.id,
                    },
                    html: None,
                    trabalho: None,
                })
            }
        }
    }
    fn ler_pagina(
        html: &crate::tela_html::HtmlServido,
        na_pagina: (FaixaDaPesquisa, u64),
    ) -> Result<RespostaDeDocumentos, ErroRespostaDaPesquisa> {
        ler_resposta_de_documentos(html, Some(na_pagina))
    }
    fn contagem(resposta: &RespostaDeDocumentos) -> Contagem {
        match resposta {
            RespostaDeDocumentos::SemResultado => Contagem::SemResultado,
            RespostaDeDocumentos::DocumentoUnico { .. } => Contagem::Unico,
            RespostaDeDocumentos::Lista(lista) => Contagem::Lista {
                faixa: lista.faixa,
                total: lista.total_declarado,
            },
        }
    }
}
