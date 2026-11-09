//! Árvore de processo sintetizada da identidade e atribuição no SQLite.
use super::{ProcessoSimulado, html};
use crate::Estado;
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

/// Projeta abertura de unidades na forma do nó raiz interpretada pelo IES.
/// Hipótese local revisável, com estado derivado exclusivamente do SQLite.
fn situacao_da_raiz(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    atribuicao: Option<&iesei::atribuicoes::Observacao>,
) -> String {
    let ancora = |sigla: &str| -> String {
        let sigla = html(sigla);
        format!(r#"<a alt="{sigla}" title="{sigla}" class="ancoraSigla">{sigla}</a>"#)
    };
    let com_atribuicao = |sigla: &str| -> String {
        let mut texto = ancora(sigla);
        if sigla == unidade_atual
            && let Some(observada) = atribuicao
            && !observada.login_atual.is_empty()
        {
            texto.push_str(" (atribuído para ");
            texto.push_str(&ancora(&observada.login_atual));
            texto.push(')');
        }
        texto
    };
    match processo.aberto_em.as_slice() {
        [] => "Processo não possui andamentos abertos.<br />".to_owned(),
        [unidade] => format!(
            "Processo aberto somente na unidade {}.<br />",
            com_atribuicao(unidade)
        ),
        unidades => {
            let mut texto = String::from("Processo aberto nas unidades:<br />");
            for unidade in unidades {
                texto.push_str(&com_atribuicao(unidade));
                texto.push_str("<br />");
            }
            texto.push_str("<br />");
            texto
        }
    }
}

/// O leitor do IES interpreta literais JS, não executa a árvore.
fn literal_js(texto: &str) -> String {
    texto
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn arvore_do_processo(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Result<String, Estado> {
    let nos = crate::documento_arvore::nos_do_processo(banco, instalacao, cenario, &processo.id)?;
    let id = &processo.id;
    let numero = literal_js(&processo.numero);
    let atribuicao = iesei::atribuicoes::ler(banco, instalacao, cenario, id, unidade_atual)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let situacao = literal_js(&situacao_da_raiz(
        processo,
        unidade_atual,
        atribuicao.as_ref(),
    ));
    let relacionados = iesei::relacionamentos::observado(banco, instalacao, cenario, id)
        .map_err(|_| Estado::DadosInsuficientes)?;
    let barra = literal_js(&crate::processo_barra::acoes_do_processo(
        processo,
        unidade_atual,
        relacionados,
        atribuicao.is_some(),
    ));
    Ok(format!(
        r#"<html><head><script>
var Nos = new Array();
Nos[0] = new infraArvoreNo("PROCESSO","{id}",null,"controlador.php?acao=arvore_visualizar&id_procedimento={id}","ifrVisualizacao","{numero}","Processo simulado","svg/processo.svg","svg/processo.svg","svg/processo.svg",true,true,"noVisitado",null,"noVisitado","{numero}");
Nos[0].acoes = '{barra}';
Nos[0].src = '';
Nos[0].html = '{situacao}';
{nos}</script></head><body><div id="divArvore"></div><form id="frmArvore"></form></body></html>"#
    ))
}
