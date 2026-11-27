//! CLI Web de blocos: GET Controle, GET lista e, para bloco N, GET detalhe.
//! Paginação interna com um POST de formulário observado, sem JS, retry ou restauração.
use crate::acao_do_sei::AcaoDoSei;
use crate::bloco::{self, ErroBloco, TipoBloco};
use crate::bloco_detalhe::{self};
use crate::configuracao;
use crate::controle_reconhecido;
use crate::erro_web::ErroCaixa;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::transporte::{self, FalhaTransporte};
use crate::visao_do_controle::{EnvioDoControle, EnvioHttps};
use crate::{Bloco, BlocoAcao, Cli};
use serde_json::{Value, json};
pub(crate) mod criar;
mod paginacao;
mod resolucao;
pub(crate) use resolucao::ler_bloco_por_numero_via;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaLeitura {
    Transporte(#[from] FalhaTransporte),
    Controle(#[from] ErroCaixa),
    Bloco(#[from] ErroBloco),
    Detalhe(#[source] crate::bloco_detalhe::ErroBlocoDetalhe),
    Submissao(#[from] ErroSubmissaoFormulario),
    UnidadeInesperada,
    HtmlDeBlocosInvalido,
    IdentidadeDoBlocoAmbigua,
    #[error("nao_confirmado")]
    NaoConfirmado,
}
/// Lista ou detalhe de bloco, já tipados.
pub(crate) enum Leitura {
    Lista(crate::bloco::ListaDeBlocos),
    Detalhe(bloco_detalhe::BlocoDetalhado),
}

/// O que a leitura de blocos observou.
pub(crate) struct Lido {
    pub(crate) unidade: String,
    pub(crate) requisicoes: usize,
    pub(crate) leitura: Leitura,
    escopo_criacao: bool,
}

/// Transporte injetável, mesmo caminho da produção; cada chamada faz um GET.
fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
    envio: &mut V,
) -> Result<Value, FalhaLeitura> {
    let Lido {
        unidade,
        requisicoes,
        leitura,
        ..
    } = ler_tipado_via(sessao, unidade_esperada, tipo, numero, pagina, envio)?;
    Ok(match leitura {
        Leitura::Detalhe(detalhe) => {
            json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"bloco":detalhe})
        }
        Leitura::Lista(lista) => {
            json!({"unidade":unidade,"origem":"web","requisicoes":requisicoes,"lista":lista})
        }
    })
}

/// A mesma leitura, tipada: o `bloco incluir` reaproveita o detalhe.
pub(crate) fn ler_tipado_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
    envio: &mut V,
) -> Result<Lido, FalhaLeitura> {
    let mut requisicoes = 0usize;
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| {
            requisicoes += 1;
            envio.get(url, cookie)
        },
    )?;
    let unidade = controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada
        .is_some_and(|v| v.split_whitespace().collect::<Vec<_>>().join(" ") != unidade)
    {
        return Err(FalhaLeitura::UnidadeInesperada);
    }
    let url_lista = bloco::link_do_menu(&controle.instalacao, &controle.url, &controle.html, tipo)?;
    let cookie = sessao.cookie().valor()?;
    requisicoes += 1;
    let resposta = envio.get(&url_lista, cookie)?;
    let codificacao = codificacao_da_tela(&resposta.mime);
    let html = transporte::decodificar_resposta(resposta)?;
    let escopo_criacao = tipo == TipoBloco::Assinatura && bloco::escopo_criacao(&html);
    let mut lista = bloco::ler_lista(&html, tipo)?;
    if pagina == Some(2) && tipo == TipoBloco::Interno {
        lista = paginacao::ler_pagina_dois(
            &lista,
            &html,
            &url_lista,
            codificacao,
            &controle.instalacao,
            sessao,
            envio,
        )?;
        requisicoes += 1;
    }
    if pagina == Some(1)
        && lista
            .lista
            .faixa_observada
            .as_ref()
            .is_some_and(|f| f.inicio != 1)
    {
        return Err(FalhaLeitura::Bloco(
            ErroBloco::PaginaDeBlocosNaoImplementada,
        ));
    }
    if let Some(numero) = numero {
        let href = lista.href_do_bloco(numero)?;
        let url = bloco::resolver_link(
            &controle.instalacao,
            &url_lista,
            href,
            AcaoDoSei::RelBlocoProtocoloListar,
            Some(numero),
        )?;
        requisicoes += 1;
        let resposta = envio.get(&url, cookie)?;
        let html = transporte::decodificar_resposta(resposta)?;
        let estado = lista
            .lista
            .itens
            .iter()
            .find(|b| b.numero == numero)
            .ok_or(FalhaLeitura::Bloco(ErroBloco::BlocoNaoEncontradoNaPagina))?
            .estado;
        let detalhe = bloco_detalhe::ler_detalhe(&html, numero, tipo, estado)
            .map_err(crate::bloco_cli::FalhaLeitura::Detalhe)?;
        return Ok(Lido {
            unidade,
            requisicoes,
            leitura: Leitura::Detalhe(detalhe),
            escopo_criacao: false,
        });
    }
    Ok(Lido {
        unidade,
        requisicoes,
        leitura: Leitura::Lista(lista.lista),
        escopo_criacao,
    })
}
#[expect(
    clippy::indexing_slicing,
    reason = "serde_json::Value: leitura ausente dá Null e a escrita é em objeto json! local; projeção de saída a substituir em #733"
)]
fn executar_leitura(
    cli: &Cli,
    tipo: TipoBloco,
    numero: Option<&str>,
    pagina: Option<u32>,
) -> anyhow::Result<()> {
    if pagina.is_some_and(|p| p > 2) || (pagina == Some(2) && tipo != TipoBloco::Interno) {
        return Err(FalhaLeitura::Bloco(ErroBloco::PaginaDeBlocosNaoImplementada).into());
    }
    let arquivos = configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let valor = if let Some(numero) = numero {
        let lido =
            ler_bloco_por_numero_via(&sessao, cli.unidade.as_deref(), numero, &mut EnvioHttps)?;
        let Leitura::Detalhe(detalhe) = lido.leitura else {
            return Err(FalhaLeitura::HtmlDeBlocosInvalido.into());
        };
        json!({"unidade":lido.unidade,"origem":"web","requisicoes":lido.requisicoes,"bloco":detalhe})
    } else {
        ler_via(
            &sessao,
            cli.unidade.as_deref(),
            tipo,
            numero,
            pagina,
            &mut EnvioHttps,
        )?
    };
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else if let Some(bloco) = valor.get("bloco") {
        println!(
            "IES: bloco {} ({}) — {} item(ns) observados; {} GET(s).",
            bloco["numero"],
            bloco["tipo"].as_str().unwrap_or("não identificado"),
            bloco["itens"].as_array().map_or(0, Vec::len),
            valor["requisicoes"]
        );
    } else {
        println!(
            "IES: blocos {}: {} na página observada de {} declarados; {} requisição(ões).",
            tipo.rotulo(),
            valor["lista"]["itens"].as_array().map_or(0, Vec::len),
            valor["lista"]["total_declarado"],
            valor["requisicoes"]
        );
        if let Some(itens) = valor["lista"]["itens"].as_array() {
            for item in itens {
                println!(
                    "{} | {} | {}",
                    item["numero"], item["estado"], item["descricao"]
                );
            }
        }
    }
    Ok(())
}
pub(crate) fn executar(cli: &Cli, bloco: &Bloco) -> anyhow::Result<()> {
    match (&bloco.numero, &bloco.acao) {
        (
            None,
            Some(BlocoAcao::Criar {
                descricao,
                confirmar,
            }),
        ) => criar::executar(cli, descricao, *confirmar),
        (None, Some(BlocoAcao::Listar { tipo, pagina })) => {
            executar_leitura(cli, *tipo, None, *pagina)
        }
        (
            None,
            Some(BlocoAcao::Incluir {
                bloco,
                documento,
                processo,
                confirmar,
            }),
        ) => crate::bloco_incluir_cli::executar(
            cli,
            &bloco.to_string(),
            documento.as_ref(),
            processo.as_deref(),
            *confirmar,
        ),
        (Some(numero), None) => {
            executar_leitura(cli, TipoBloco::Assinatura, Some(&numero.to_string()), None)
        }
        (None, None) => {
            crate::esquema_cli::show_noun_help("bloco");
            Ok(())
        }
        (Some(_), Some(_)) => Err(UsoDoBloco::NumeroComSubcomando.into()),
    }
}

/// Combinação de argumentos que o clap não recusa sozinho.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum UsoDoBloco {
    NumeroComSubcomando,
}
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[test]
    fn cli_aceita_listar_tipo_e_pagina_e_numero() {
        assert!(
            Cli::try_parse_from([
                "ies", "bloco", "listar", "--tipo", "interno", "--pagina", "1"
            ])
            .is_ok()
        );
        assert!(Cli::try_parse_from(["ies", "bloco", "2195"]).is_ok());
        assert!(Cli::try_parse_from(["ies", "bloco", "listar", "--tipo", "desconhecido"]).is_err());
        assert!(Cli::try_parse_from(["ies", "bloco", "listar", "--pagina", "0"]).is_err());
    }
}

#[cfg(test)]
#[path = "bloco_cli/tests_fluxo.rs"]
mod tests_fluxo;
