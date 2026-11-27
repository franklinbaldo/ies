//! Criação de grupo por formulário servido e releitura independente.
use super::{Grupo, Membro, abrir_lista_via, conferir_unidade_publicada, ler_pagina, obter_via};
use crate::formulario_web::{RespostaDoEnvio, TelaServida, enviar_submissao_via};
use crate::transporte::{decodificar_resposta, sessao::ReferenciasDaSessao};
use crate::visao_do_controle::EnvioDoControle;
use anyhow::Context;
use serde::Serialize;
use std::collections::HashSet;

mod formulario;
mod identidade;
mod membros;

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
enum ErroCriar {
    CharsetNaoSuportado,
    IdentidadeNaoConfirmada,
    IdentidadeJaObservada,
    GrupoDiferenteDoPedido,
}

#[derive(Serialize)]
#[serde(tag = "estado", rename_all = "snake_case")]
pub(super) enum Criacao {
    Previa {
        unidade: String,
        nome: String,
        descricao: String,
        membros: Vec<Membro>,
    },
    Criado {
        unidade: String,
        grupo: Grupo,
    },
}

pub(super) struct Pedido<'a> {
    pub(super) nome: &'a str,
    pub(super) descricao: &'a str,
    pub(super) unidades: &'a [String],
    pub(super) confirmar: bool,
}

fn limpo(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn criar_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    pedido: &Pedido<'_>,
    envio: &mut V,
) -> anyhow::Result<Criacao> {
    if pedido.confirmar {
        crate::origem_de_treinamento::exigir_origem(sessao.origem())?;
    }
    let nome = limpo(pedido.nome);
    anyhow::ensure!(!nome.is_empty(), "Nome de grupo vazio");
    let descricao = limpo(pedido.descricao);
    let (unidade, instalacao, lista) = abrir_lista_via(sessao, unidade_esperada, envio)?;
    conferir_unidade_publicada(&lista.html, &unidade)?;
    let (_, antes) = ler_pagina(&lista.html)?;
    let novo = crate::unidade_lupa::botao_novo(&lista.html, &lista.url, sessao.origem())?;
    let cookie = sessao.cookie().valor()?;
    let resposta = envio.get(&novo, cookie)?;
    let codificacao = crate::pagina_do_controle::codificacao_da_tela(&resposta.mime)
        .ok_or(ErroCriar::CharsetNaoSuportado)?;
    let html = decodificar_resposta(resposta)?;
    conferir_unidade_publicada(&html, &unidade)?;
    let tela = TelaServida {
        url: &novo,
        html: &html,
        codificacao,
    };
    let membros = membros::resolver(&instalacao, &tela, cookie, pedido.unidades, envio)?;
    let post = formulario::preparar(&instalacao, &tela, &unidade, &nome, &descricao, &membros)?;
    if !pedido.confirmar {
        return Ok(Criacao::Previa {
            unidade,
            nome,
            descricao,
            membros,
        });
    }
    // O único POST mutador. Um 302 é dado, não instrução de navegação.
    let resposta = enviar_submissao_via(post, sessao.cookie(), |p, c| envio.post(p, c)).context(
        "Criação não confirmada após tentativa de envio; confira o grupo antes de repetir",
    )?;
    let RespostaDoEnvio::Redirecionamento(redir) = resposta else {
        return Err(ErroCriar::IdentidadeNaoConfirmada.into());
    };
    let id = identidade::id_publicado(&redir).ok_or(ErroCriar::IdentidadeNaoConfirmada)?;
    if antes.iter().any(|l| l.item.id == id) {
        return Err(ErroCriar::IdentidadeJaObservada.into());
    }
    // Obter reabre Controle e lista: consulta independente do Location.
    // Se a página observada não contiver o ID, a criação é inconclusiva.
    let (unidade, grupo) = obter_via(sessao, Some(&unidade), id, envio)
        .context("Criação não confirmada por releitura; confira antes de repetir")?;
    let esperados: HashSet<_> = membros.iter().map(|m| m.id.as_str()).collect();
    let relidos: HashSet<_> = grupo.unidades.iter().map(|m| m.id.as_str()).collect();
    if limpo(&grupo.nome).to_lowercase() != nome.to_lowercase()
        || limpo(&grupo.descricao) != descricao
        || esperados != relidos
    {
        return Err(ErroCriar::GrupoDiferenteDoPedido.into());
    }
    Ok(Criacao::Criado { unidade, grupo })
}

#[cfg(test)]
mod tests;
