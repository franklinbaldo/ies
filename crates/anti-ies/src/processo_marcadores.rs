//! Gerenciamento de marcadores projetado de uma observação SQLite, só GET.
 //! BDD-255-03: leitura idempotente da CLI; nenhum POST de escrita simulada.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::marcadores::{Marcador, Observacao};
use rusqlite::Connection;
use std::fmt::Write as _;

fn cabecalho(html: &mut String, nup: &str, id: &str, observacao: &Observacao) {
    let nup = escapar(nup);
    let assinatura = crate::protocolo_sintetico::INFRA_HASH;
    write!(
        html,
        "<!doctype html><html><head><title>SEI - Marcadores do Processo {nup}</title></head><body>\
         <form id='frmGerenciarMarcador' method='post' \
         action='controlador.php?acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&infra_hash={assinatura}'>\
         <table id='tblMarcadores' summary='Marcadores'>\
         <caption>Marcadores ({} {}):</caption>\
         <tr><th><a id='lnkInfraCheck'></a></th>\
         <th><div class='infraDivRotuloOrdenacao'>Marcador</div></th>\
         <th>Texto</th>\
         <th><div class='infraDivRotuloOrdenacao'>Usuário</div></th>\
         <th><div class='infraDivRotuloOrdenacao'>Data/Hora</div></th>\
         <th>Ações</th></tr>",
        observacao.total_declarado,
        if observacao.total_declarado == 1 { "registro" } else { "registros" }
    ).expect("String");
}

fn linha(html: &mut String, m: &Marcador) {
    let (id, nome, cor, data) = (
        escapar(&m.id), escapar(&m.nome), escapar(&m.cor),
        escapar(&m.data_hora)
    );
    let texto = m.texto.as_deref().map(escapar).unwrap_or_default();
    let usuario = m.usuario.as_deref().map_or_else(String::new, |nome| {
        let nome = escapar(nome);
        format!("<a class='ancoraSigla'>{nome}</a>")
    });
    write!(
        html,
        "<tr class='infraTrClara'>\
         <td><a id='lnkInfraID-{id}' name='ID-{id}'></a>\
         <input type='checkbox' class='infraCheckbox' value='{id}' title='{nome}'></td>\
         <td><img src='svg/marcador_{cor}.svg' title='{nome}'>{nome}</td>\
         <td id='tdAndamentoMarcador{id}'>{texto}</td>\
         <td>{usuario}</td><td>{data}</td><td></td></tr>"
    ).expect("String");
}

/// Responde à tela de leitura cuja ação foi publicada na barra da árvore.
/// Sem observação não há afirmação de lista vazia nem rota de escrita.
///
/// # Cautela
/// O HTML local é hipótese estrutural, não transcrição de uma cassette.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let processo = match iesei::processos::ler(banco, origem, cenario, id) {
        Ok(Some(processo)) => processo,
        Ok(None) => return Resposta::nova(Estado::DadosInsuficientes),
        Err(_) => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let unidade = match crate::processo_envio::unidade_da_sessao(banco, origem, cenario, &processo) {
        Ok(unidade) => unidade,
        Err(estado) => return Resposta::nova(estado),
    };
    if !processo.aberto_em.contains(&unidade) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let observacao = match iesei::marcadores::ler(banco, origem, cenario, id, &unidade) {
        Ok(Some(observacao)) => observacao,
        _ => return Resposta::nova(Estado::DadosInsuficientes),
    };
    let mut html = String::new();
    cabecalho(&mut html, &processo.numero, id, &observacao);
    for marcador in &observacao.itens {
        linha(&mut html, marcador);
    }
    html.push_str("</table></form></body></html>");
    Resposta::html(html)
}
