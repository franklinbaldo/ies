//! Acompanhamento Especial com estado SQLite por unidade e processo.
//! HTML e token são projeções sintéticas, nunca cópias da cassette.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;
use std::fmt::Write as _;

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let acao = parametro_unico(alvo, "acao");
    let permitido = matches!(
        (metodo, acao),
        ("GET", Some("acompanhamento_gerenciar")) | ("POST", Some("acompanhamento_excluir"))
    );
    if !permitido {
        return Resposta::nova(Estado::MetodoNaoPermitido);
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
    let Ok(Some(observacao)) = iesei::acompanhamentos::ler(banco, origem, cenario, id, &unidade)
    else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let gerenciar = format!(
        "controlador.php?acao=acompanhamento_gerenciar&id_procedimento={id}&infra_hash={hash}"
    );
    if metodo == "POST" {
        // O formulário observado envia só o oculto reescrito pelo handler.
        let Some(alvo) = corpo
            .strip_prefix("hdnInfraItemId=")
            .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
        else {
            return Resposta::nova(Estado::RequisicaoInvalida);
        };
        if !observacao.itens.iter().any(|item| item.id == alvo) {
            return Resposta::nova(Estado::RequisicaoInvalida);
        }
        return match iesei::acompanhamentos::retirar(banco, origem, cenario, id, &unidade, alvo) {
            Ok(true) => Resposta::redirecionar(gerenciar),
            Ok(false) => Resposta::nova(Estado::RequisicaoInvalida),
            Err(_) => Resposta::nova(Estado::DadosInsuficientes),
        };
    }
    if observacao.itens.is_empty() {
        // Hipótese sintética: lista comprovadamente vazia redireciona.
        return Resposta::redirecionar(format!(
            "controlador.php?acao=acompanhamento_cadastrar&id_procedimento={id}&infra_hash={hash}"
        ));
    }

    let destino = format!(
        "controlador.php?acao=acompanhamento_excluir&acao_origem=acompanhamento_gerenciar&id_procedimento={id}&infra_hash={hash}"
    );
    let mut html = format!(
        "<!doctype html><html><head><script>         function acaoExcluir(id,desc){{if(confirm('Excluir?')){{         document.getElementById('hdnInfraItemId').value=id;         document.getElementById('frmGerenciarAcompanhamento').action='{destino}';         document.getElementById('frmGerenciarAcompanhamento').submit();         }}}}</script></head><body>         <form id='frmGerenciarAcompanhamento' method='post' action='{}'>         <input type='hidden' name='hdnInfraItemId' id='hdnInfraItemId' value=''>         <table summary='Tabela de Acompanhamentos.'>         <tr><th></th><th>Grupo</th><th>Observação</th>         <th>Usuário</th><th>Data</th><th>Ações</th></tr>",
        escapar(&gerenciar)
    );
    for item in observacao.itens {
        write!(
            html,
            "<tr><td><input type='checkbox' value='{}'></td>             <td>{}</td><td>{}</td><td></td><td></td>             <td><a onclick=\"acaoExcluir('{}','');\">Excluir</a></td></tr>",
            escapar(&item.id),
            escapar(&item.grupo),
            escapar(&item.observacao),
            escapar(&item.id),
        )
        .expect("formatação String");
    }
    html.push_str("</table></form></body></html>");
    Resposta::html(html)
}
