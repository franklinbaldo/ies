//! Renderização da visão detalhada sintética a partir do SQLite, nunca HTML congelado.
use iesei::detalhada_cassettes::ControleDetalhado;
use std::fmt::Write as _;

pub(crate) fn html(controle: &ControleDetalhado) -> Option<String> {
    let tabela = &controle.processos;
    if !tabela.pagina_verificavel() {
        return None;
    }
    let total = tabela.total_declarado?;
    let legenda = tabela.faixa_observada.map_or_else(
        || format!("Processos ({total} registros)"),
        |faixa| {
            format!(
                "Processos ({total} registros - {} a {})",
                faixa.inicio(),
                faixa.fim()
            )
        },
    );
    let mut html = format!(
        "<html><body><a id='lnkInfraUnidade'>{}</a>\
         <input id='hdnTipoVisualizacao' value='D'>\
         <input id='hdnRecebidosHashCriterios' value='r'>\
         <input id='hdnGeradosHashCriterios' value='g'>",
        crate::escapar(controle.unidade.as_str())
    );
    if let Some(faixa) = tabela.faixa_observada {
        // Índice apenas ilustrativo: página posterior não significa página integral.
        if faixa.inicio() > 1 {
            write!(html, "<input id='hdnDetalhadoPaginaAtual' value='1'>").ok()?;
        }
    }
    write!(
        html,
        "<table id='tblProcessosDetalhado'><caption>{legenda}:</caption>\
         <tr><th></th><th>Processo</th></tr>"
    )
    .ok()?;
    for linha in &tabela.linhas {
        let classe = match (linha.visualizado, linha.sigiloso) {
            (Some(true), Some(false)) => " class='processoVisualizado'",
            (Some(false), Some(false)) => " class='processoNaoVisualizado'",
            (Some(true), Some(true)) => " class='processoVisualizadoSigiloso'",
            (Some(false), Some(true)) => " class='processoNaoVisualizadoSigiloso'",
            (None, None) => "",
            _ => return None,
        };
        write!(
            html,
            "<tr id='P{}'><td></td><td><a{classe} href='controlador.php?acao=procedimento_trabalhar&amp;id_procedimento={}'>{}</a></td></tr>",
            crate::escapar(&linha.id),
            crate::escapar(&linha.id),
            crate::escapar(&linha.numero)
        )
        .ok()?;
    }
    html.push_str("</table></body></html>");
    Some(html)
}
