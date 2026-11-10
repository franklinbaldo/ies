//! Tela de trabalho sintética derivada da exigência tipada no SQLite.
use super::{ProcessoSimulado, html};

pub(super) fn trabalho_do_processo(
    processo: &ProcessoSimulado,
    unidade_atual: &str,
    validacao: Option<iesei::validacao_acesso::Validacao>,
) -> String {
    let id = &processo.id;
    let exige_senha = matches!(validacao, Some(iesei::validacao_acesso::Validacao::Exigida));
    // Hipotese estrutural BDD-633-02: a modal foi observada no GO, mas
    // a pagina original da modal e o formulario nao foram capturados.
    let src = if exige_senha {
        String::new()
    } else {
        format!(
            "controlador.php?acao=procedimento_visualizar&amp;id_procedimento={id}&amp;infra_hash={}",
            crate::protocolo_sintetico::INFRA_HASH
        )
    };
    let modal = if exige_senha {
        format!(
            "<script>function inicializar(){{infraAbrirJanelaModal('controlador.php?acao=usuario_validar_acesso&acao_origem=procedimento_trabalhar&id_procedimento={id}&infra_hash={}',500,300,true,'finalizar');}}</script>",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    // A unidade sem permissao de abrir a arvore nao oferece inclusao.
    let incluir = if !exige_senha && processo.pode_incluir_documento(unidade_atual) {
        format!(
            "<a id='incluirDocumento' href='controlador.php?acao=documento_escolher_tipo&amp;id_procedimento={id}&amp;infra_hash={}'>Incluir Documento</a>",
            crate::protocolo_sintetico::INFRA_HASH
        )
    } else {
        String::new()
    };
    format!(
        "<html><head><title>SEI - Processo</title></head><body>\\
         <a id='lnkInfraUnidade'>{}</a>\\
         <iframe id='ifrArvore' name='ifrArvore' src='{src}'></iframe>\\
         <iframe id='ifrVisualizacao' src='about:blank'></iframe>\\
         {incluir}{modal}</body></html>",
        html(unidade_atual),
    )
}

