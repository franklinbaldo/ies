//! Controle resumido R: HTML tipado e sem recuperação de conteúdo externo.
use crate::escapar;
use iesei::banco_cassettes::{ControleResumido, Linha};
use iesei::observacao::TabelaObservada;
use std::fmt::Write as _;

fn tabela_html(id: &str, tabela: &TabelaObservada<Linha>, destino: &mut String) -> Option<()> {
    if !tabela.pagina_verificavel() {
        return None;
    }
    // O parser do Controle precisa de um total publicado. Ausência não é zero.
    let total = tabela.total_declarado?;
    let mut legenda = format!("{total} registros");
    if let Some(faixa) = tabela.faixa_observada {
        write!(legenda, " - {} a {}", faixa.inicio(), faixa.fim()).ok()?;
    }
    write!(destino, "<table id='{id}'><caption>{legenda}:</caption>").ok()?;
    for linha in &tabela.linhas {
        let classe = match (linha.visualizado, linha.sigiloso) {
            (true, false) => "processoVisualizado",
            (false, false) => "processoNaoVisualizado",
            (true, true) => "processoVisualizadoSigiloso",
            (false, true) => "processoNaoVisualizadoSigiloso",
        };
        // A identidade da linha é navegável na forma observada do Controle R.
        // Sem href, a CLI abandona o ID observado e tenta outra pesquisa.
        let id_linha = escapar(&linha.id);
        write!(
            destino,
            "<tr id='P{id_linha}'><td><a class='{classe}' href='controlador.php?acao=procedimento_trabalhar&amp;acao_origem=procedimento_controlar&amp;acao_retorno=procedimento_controlar&amp;id_procedimento={id_linha}&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={}'>{}</a></td></tr>",
            crate::protocolo_sintetico::INFRA_HASH,
            escapar(&linha.numero)
        )
        .ok()?;
    }
    destino.push_str("</table>");
    Some(())
}

pub(crate) fn controle_html(controle: &ControleResumido) -> Option<String> {
    // A pesquisa e o unico POST da leitura de documento, sem criar objeto.
    // Assinaturas aqui sao placeholders locais, nunca tokens de SEI.
    let formulario = format!(
        r#"<form id="frmProtocoloPesquisaRapida" method="post" action="controlador.php?acao=protocolo_pesquisa_rapida&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={}"><input type="text" id="txtPesquisaRapida" name="txtPesquisaRapida" /><img src="svg/pesquisa_rapida.svg" onclick="document.getElementById('frmProtocoloPesquisaRapida').submit();" /></form>"#,
        crate::protocolo_sintetico::INFRA_HASH
    );
    // Link do menu para pesquisa documental, distinto da pesquisa rápida.
    let menu_pesquisa = format!(
        "<a link='protocolo_pesquisar' href='controlador.php?acao=protocolo_pesquisar&amp;infra_sistema=100000100&amp;infra_unidade_atual=110000001&amp;infra_hash={}'>Pesquisa</a>",
        crate::protocolo_sintetico::INFRA_HASH
    );
    let mut html = format!(
        "<html><body>{formulario}{menu_pesquisa}<a id='lnkInfraUnidade'>{}</a><input id='hdnTipoVisualizacao' value='R'>",
        escapar(controle.unidade.as_str())
    );
    tabela_html("tblProcessosRecebidos", &controle.recebidos, &mut html)?;
    tabela_html("tblProcessosGerados", &controle.gerados, &mut html)?;
    html.push_str("</body></html>");
    Some(html)
}
