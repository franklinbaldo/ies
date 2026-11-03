//! Fixtures SINTÉTICAS com a estrutura observada na `UnB` 4.0.9 e em GO
//! 5.0.3 (03/11/2026): o link do histórico nas duas formas da árvore e a
//! página do histórico (`R` e `P`). Hosts, hashes, ids, nomes e datas
//! inventados; nenhum valor servido.
use std::fmt::Write as _;

pub(crate) const HASH_HISTORICO: &str =
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
pub(crate) const HASH_FORM: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
pub(crate) const HASH_ATUALIZAR: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";
const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

/// O link relativo do histórico, como a árvore o publica.
pub(crate) fn link_do_historico(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_consultar_historico&acao_origem=arvore_visualizar&acao_retorno=arvore_visualizar&id_procedimento={id}&arvore=1&{CONTEXTO}&infra_hash={HASH_HISTORICO}"
    )
}

/// `action` do `form#frmProcedimentoHistorico`.
pub(crate) fn action_do_historico(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_consultar_historico&acao_origem=procedimento_consultar_historico&arvore=1&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_FORM}"
    )
}

/// Forma `UnB`: função no `script` inline e `onclick` sem argumento.
pub(crate) fn forma_unb(url: &str) -> (String, String) {
    let script = format!(
        "  function consultarAndamento(){{\n    parent.document.getElementById('ifrVisualizacao').src = '{url}';\n    var no = objArvore.getNoSelecionado();\n  }}\n"
    );
    let corpo = "<div id=\"divConsultarAndamento\">\n        <a href=\"#\" style=\"cursor:pointer;\" onclick=\"consultarAndamento();\"><img src=\"svg/processo_andamentos.svg\" alt=\"Consultar Andamento\" />&nbsp;Consultar Andamento</a>\n      </div>\n".to_owned();
    (script, corpo)
}

/// Forma GO: a URL no `onclick`; a função não é definida.
pub(crate) fn forma_go(url: &str) -> (String, String) {
    let corpo = format!(
        "<div id=\"divConsultarAndamento\">\n        <a href=\"#\" style=\"cursor:pointer;\" onclick=\"consultarAndamento('{url}');\"><img src=\"svg/processo_andamentos.svg\" alt=\"Consultar Andamento\" />&nbsp;Consultar Andamento</a>\n      </div>\n"
    );
    (String::new(), corpo)
}

/// Insere `(script, corpo)` numa árvore sintética (antes de `</script>` e
/// de `</body>`), com o CSS `#divConsultarAndamento` que as duas servem.
pub(crate) fn com_link(arvore: &str, partes: &[(String, String)]) -> String {
    let mut script = String::new();
    let mut corpo = String::new();
    for (s, c) in partes {
        script.push_str(s);
        corpo.push_str(c);
    }
    arvore
        .replacen(
            "<html><head>",
            "<html><head><style>\n#divConsultarAndamento {\n}\n#divConsultarAndamento a {\n}\n</style>",
            1,
        )
        .replacen("</script>", &format!("{script}</script>"), 1)
        .replacen("</body>", &format!("{corpo}</body>"), 1)
}

/// Página do histórico com `linhas` andamentos e a legenda de `total`.
pub(crate) fn pagina_do_historico(
    nup: &str,
    id: &str,
    tipo: char,
    total: u64,
    linhas: usize,
) -> String {
    let (outro, texto) = if tipo == 'R' {
        ('P', "Ver histórico completo")
    } else {
        ('R', "Ver histórico resumido")
    };
    let palavra = if total == 1 { "registro" } else { "registros" };
    let mut tabela = String::new();
    for i in 0..linhas {
        let _ = write!(
            tabela,
            "\n<!-- {i} -->\n<tr class=\"andamentoConcluido\">\n<td align=\"center\" valign=\"top\">0{}/01/2026 10:00</td>\n<td align=\"center\"  valign=\"top\"><a alt=\"UNIDADE {i}\" title=\"UNIDADE {i}\" class=\"ancoraSigla\">U{i}</a></td>\n<td align=\"center\"  valign=\"top\"><a alt=\"Pessoa {i}\" title=\"Pessoa {i}\" class=\"ancoraSigla\">p{i}</a></td>\n<td valign=\"top\">Andamento sintetico {i}<span style=\"line-height:.1em\"><br /></span></td></tr>",
            i % 9 + 1
        );
    }
    format!(
        "<!DOCTYPE html>\n<html lang=\"pt-br\" >\n<head >\n<meta http-equiv=\"Content-Type\" content=\"text/html; charset=iso-8859-1\" />\n<title>SEI - Histórico do Processo {nup}&nbsp;&nbsp;</title>\n<script type=\"text/javascript\" charset=\"iso-8859-1\" >\n<!--//--><![CDATA[//><!--\nfunction inicializar(){{\n  if (document.getElementById('btnAtualizarAndamento')!=null){{\n    document.getElementById('btnAtualizarAndamento').focus();\n  }}\n}}\nfunction verHistorico(valor){{\n  document.getElementById('hdnTipoHistorico').value = valor;\n  document.getElementById('frmProcedimentoHistorico').submit();\n}}\n//--><!]]>\n</script>\n</head>\n<body onload=\"inicializar();\"  >\n<div id=\"divInfraBarraLocalizacao\" class=\"infraBarraLocalizacao\">Histórico do Processo {nup}&nbsp;&nbsp;</div>\n<form id=\"frmProcedimentoHistorico\" method=\"post\" action=\"{action}\">\n  <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"1\" />\n<div id=\"divInfraBarraComandosSuperior\" class=\"infraBarraComandos\">\n<button tabindex=\"451\" type=\"button\" accesskey=\"A\" id=\"btnAtualizarAndamento\" name=\"btnAtualizarAndamento\" value=\"Atualizar Andamento\" onclick=\"location.href='controlador.php?acao=procedimento_atualizar_andamento&acao_origem=arvore_visualizar&id_procedimento={id}&{CONTEXTO}&infra_hash={HASH_ATUALIZAR}';\" class=\"infraButton\">Atualizar Andamento</button>\n</div>\n<div id=\"divInfraAreaDados\" class=\"infraAreaDados\">\n<a id=\"ancTipoHistorico\" href=\"#\" onclick=\"verHistorico('{outro}');\" class=\"ancoraPadraoPreta\">{texto}</a>&nbsp;</div>\n<div id=\"divInfraAreaPaginacaoSuperior\" class=\"infraAreaPaginacao\">\n</div>\n<div id=\"divInfraAreaTabela\" class=\"infraAreaTabela\" >\n<table id=\"tblHistorico\" width=\"99%\" class=\"infraTable\" summary=\"Histórico de Andamentos\">\n<caption class=\"infraCaption\">Lista de Andamentos ({total} {palavra}):</caption><tr><th class=\"infraTh\">Data/Hora</th><th class=\"infraTh\">Unidade</th><th class=\"infraTh\">Usuário</th><th class=\"infraTh\">Descrição</th></tr>{tabela}</table>\n</div>\n<input type=\"hidden\" id=\"hdnInfraNroItens\" name=\"hdnInfraNroItens\" value=\"{linhas}\" />\n<input type=\"hidden\" id=\"hdnInfraItemId\" name=\"hdnInfraItemId\" value=\"\" />\n<input type=\"hidden\" id=\"hdnInfraSelecoes\" name=\"hdnInfraSelecoes\" value=\"Infra\" />\n<input type=\"hidden\" id=\"hdnInfraPaginaAtual\" name=\"hdnInfraPaginaAtual\" value=\"0\" />\n<input type=\"hidden\" id=\"hdnInfraHashCriterios\" name=\"hdnInfraHashCriterios\" value=\"abcdef0123456789abcdef0123456789\" />\n  <input type=\"hidden\" id=\"hdnTipoHistorico\" name=\"hdnTipoHistorico\" value=\"{tipo}\" />\n</form>\n<input type=\"hidden\" id=\"hdnInfraPrefixoCookie\" name=\"hdnInfraPrefixoCookie\" value=\"ORG_SEI_pessoa\" />\n</body>\n</html>\n",
        action = action_do_historico(id),
    )
}

/// Corpo esperado do POST `verHistorico('P')`: ocultos na ordem servida,
/// `hdnTipoHistorico=P`, sem o botão de escrita.
pub(crate) fn corpo_ver_completo(linhas: usize) -> String {
    format!(
        "hdnInfraTipoPagina=1&hdnInfraNroItens={linhas}&hdnInfraItemId=&hdnInfraSelecoes=Infra&hdnInfraPaginaAtual=0&hdnInfraHashCriterios=abcdef0123456789abcdef0123456789&hdnTipoHistorico=P"
    )
}
