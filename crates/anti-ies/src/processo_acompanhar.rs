//! GET de Acompanhamento Especial da unidade, derivado do SQLite efêmero.
//! BDD-348-01/03: tabela de leitura sem ação de exclusão nem POST.
//! O HTML projetado é hipótese estrutural, não reprodução de uma cassette.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

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
    if parametro_unico(alvo, "acao") != Some("acompanhamento_gerenciar") {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(processo)) = iesei::processos::ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let unidade = match crate::processo_envio::unidade_da_sessao(banco, origem, cenario, &processo)
    {
        Ok(unidade) => unidade,
        Err(estado) => return Resposta::nova(estado),
    };
    if !processo.aberto_em.contains(&unidade) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(observacao)) =
        iesei::acompanhamentos::ler(banco, origem, cenario, id, &unidade)
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    if observacao.itens.is_empty() {
        // Só a observação explícita permite este 302 sintético de lista vazia.
        return Resposta::redirecionar(format!(
            "controlador.php?acao=acompanhamento_cadastrar&id_procedimento={id}&infra_hash={hash}"
        ));
    }
    let mut html = String::from(
        "<!doctype html><html><body>         <table summary='Tabela de Acompanhamentos.'>         <tr><th></th><th>Grupo</th><th>Observação</th>         <th>Usuário</th><th>Data</th><th>Ações</th></tr>",
    );
    for item in observacao.itens {
        write!(
            html,
            "<tr><td><input type='checkbox' value='{}'></td>             <td>{}</td><td>{}</td><td></td><td></td><td>Consultar</td></tr>",
            escapar(&item.id),
            escapar(&item.grupo),
            escapar(&item.observacao),
        )
        .expect("formatação String");
    }
    html.push_str("</table></body></html>");
    Resposta::html(html)
}
