//! Telas SINTÉTICAS da busca de processo, com a estrutura observada na
//! `UnB` 4.0.9 de treinamento (03/11/2026): menu `protocolo_pesquisar` no
//! Controle, `frmPesquisaProtocolo`, barra "Exibindo I - F de N", linhas com
//! `a.arvore` e `a.protocoloNormal` (com `class` repetido), `navegar()` e
//! aviso `pesquisaSemResultado`. Nenhum valor real: hosts, hashes, IDs e
//! números são inventados.

pub(crate) const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
pub(crate) const UNIDADE: &str = "UNIDADE TESTE";
pub(crate) const CONTEXTO: &str = "infra_sistema=100000100&infra_unidade_atual=110000001";

pub(crate) fn hash(c: char) -> String {
    c.to_string().repeat(64)
}

pub(crate) fn url_controle() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_controlar&{CONTEXTO}&infra_hash={}",
        hash('a')
    )
}

/// O `href` do menu como servido (com `&` cru, como na `UnB`).
pub(crate) fn href_do_menu() -> String {
    format!(
        "controlador.php?acao=protocolo_pesquisar&{CONTEXTO}&infra_hash={}",
        hash('b')
    )
}

/// A URL que o GET da tela de pesquisa deve usar.
pub(crate) fn url_da_tela() -> String {
    format!("{ORIGEM}/{}", href_do_menu())
}

/// A URL do POST (action sem o fragmento `#ancoraBarraPesquisa`).
pub(crate) fn url_do_envio() -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=protocolo_pesquisar&acao_origem=protocolo_pesquisar&{CONTEXTO}&infra_hash={}",
        hash('c')
    )
}

pub(crate) fn menu(href: &str) -> String {
    format!(
        "<li><a style=\"padding-left:5px\" link=\"protocolo_pesquisar\" href=\"{href}\">\
         <img src=\"menu/pesquisa.svg\"/><span>Pesquisa</span></a></li>"
    )
}

/// Controle na visão resumida, reconhecível, com o `menu` dado.
pub(crate) fn controle(menu: &str) -> String {
    format!(
        "<html><body><ul id=\"infraMenu\">{menu}\
         <li><a href=\"controlador.php?acao=protocolo_pesquisa_rapida&{CONTEXTO}&infra_hash={}\">R</a></li></ul>\
         <a id=\"lnkInfraUnidade\">{UNIDADE}</a>\
         <table id=\"tblProcessosRecebidos\"><caption>Processos (1 registros - 1 a 1):</caption>\
         <tr id=\"P1\"><td><a class=\"processoVisualizado\">R-0001</a></td></tr></table>\
         <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></body></html>",
        hash('f')
    )
}

const SCRIPT_NAVEGAR: &str = "<script>\n  function navegar(inicio) {\n    \
    document.getElementById('hdnInicio').value = inicio;\n    \
    if (typeof(window.onSubmitForm)=='function' && !window.onSubmitForm()) {\n      return;\n    }\n    \
    document.getElementById('frmPesquisaProtocolo').submit();\n  }\n</script>";

/// `frmPesquisaProtocolo` com os controles na ordem servida; `conteudo`
/// (barra, tabela, paginador) fica dentro do formulário, antes de `hdnInicio`.
pub(crate) fn formulario(protocolo: &str, processos_marcado: bool, conteudo: &str) -> String {
    let (p, d) = if processos_marcado {
        (" checked=\"checked\"", "")
    } else {
        ("", " checked=\"checked\"")
    };
    format!(
        "<form id=\"frmPesquisaProtocolo\" name=\"frmPesquisaProtocolo\" method=\"post\" \
         onsubmit=\"return onSubmitForm();\" \
         action=\"controlador.php?acao=protocolo_pesquisar&acao_origem=protocolo_pesquisar&{CONTEXTO}&infra_hash={}#ancoraBarraPesquisa\">\
         <input type=\"hidden\" id=\"hdnInfraTipoPagina\" name=\"hdnInfraTipoPagina\" value=\"1\" />\
         <button type=\"submit\" id=\"sbmPesquisar\" name=\"sbmPesquisar\" value=\"Pesquisar\">Pesquisar</button>\
         <button type=\"button\" id=\"sbmReset\" name=\"sbmReset\" value=\"Limpar\">Limpar</button>\
         <input type=\"radio\" id=\"optProcessos\" name=\"rdoPesquisarEm\" value=\"P\"{p} />\
         <input type=\"checkbox\" id=\"chkSinConsiderarDocumentos\" name=\"chkSinConsiderarDocumentos\" value=\"S\" />\
         <input type=\"radio\" id=\"optDocumentos\" name=\"rdoPesquisarEm\" value=\"D\"{d} />\
         <input type=\"checkbox\" id=\"chkSinDocumentosGerados\" name=\"chkSinDocumentosGerados\" value=\"S\" checked=\"checked\" />\
         <input type=\"checkbox\" id=\"chkSinDocumentosRecebidos\" name=\"chkSinDocumentosRecebidos\" value=\"S\" checked=\"checked\" />\
         <input type=\"text\" id=\"q\" name=\"q\" value=\"\" />\
         <select style=\"display: none\" multiple id=\"selOrgaoPesquisa\" name=\"selOrgaoPesquisa[]\">\
         <option value=\"0\" selected=\"selected\">ORGAO</option><option value=\"9\">OUTRO</option></select>\
         <input type=\"hidden\" id=\"hdnIdUnidade\" name=\"hdnIdUnidade\" value=\"\" />\
         <input type=\"hidden\" id=\"hdnIdAssunto\" name=\"hdnIdAssunto\" value=\"\" />\
         <input type=\"hidden\" id=\"hdnIdAssinante\" name=\"hdnIdAssinante\" value=\"\" />\
         <input type=\"hidden\" id=\"hdnIdContato\" name=\"hdnIdContato\" value=\"\" />\
         <input type=\"text\" id=\"txtProtocoloPesquisa\" name=\"txtProtocoloPesquisa\" maxlength=\"50\" value=\"{protocolo}\" />\
         <input type=\"hidden\" id=\"hdnIdUsuarioGerador1\" name=\"hdnIdUsuarioGerador1\" value=\"\" />\
         <input type=\"hidden\" id=\"hdnIdUsuarioGerador2\" name=\"hdnIdUsuarioGerador2\" value=\"\" />\
         <input type=\"hidden\" id=\"hdnIdUsuarioGerador3\" name=\"hdnIdUsuarioGerador3\" value=\"\" />\
         <div id=\"conteudo\">{conteudo}</div>\
         <input type=\"hidden\" id=\"hdnInicio\" name=\"hdnInicio\" value=\"0\" /></form>",
        hash('c')
    )
}

pub(crate) fn pagina(corpo: &str) -> String {
    format!(
        "<html><head>{SCRIPT_NAVEGAR}</head><body>\
         <a id=\"lnkInfraUnidade\">{UNIDADE}</a>{corpo}</body></html>"
    )
}

/// A tela do GET do menu: `optDocumentos` marcado, sem resultado.
pub(crate) fn tela_de_pesquisa() -> String {
    pagina(&formulario("", false, ""))
}

pub(crate) fn numero(posicao: u64) -> String {
    format!("00001.{posicao:06}/2026-00")
}

/// Uma linha como na `UnB`: `a.arvore` e `a.protocoloNormal` com o mesmo
/// `href`, e `class` repetido (idêntico) no segundo.
pub(crate) fn linha(posicao: u64) -> String {
    let href = format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={}&{CONTEXTO}&infra_hash={}",
        1000 + posicao,
        hash('e')
    );
    let numero = numero(posicao);
    format!(
        "<tr class=\"pesquisaTituloRegistro\"><td colspan=\"2\" class=\"pesquisaTituloEsquerda\">\
         <a onclick=\"x(this);\" href=\"{href}\" target=\"_blank\" class=\"arvore\">\
         <img border=\"0\" src=\"svg/arvore.svg\" alt=\"Tipo N {numero}\" class=\"arvore\" /></a>\
         <span>Tipo N </span><a onclick=\"x(this);\" target=\"_blank\" class=\"protocoloNormal\" \
         href=\"{href}\" title=\"Tipo\" class=\"protocoloNormal\" tabindex=\"1\">{numero}</a></td>\
         <td class=\"pesquisaTituloDireita\"></td></tr><tr><td class=\"pesquisaSnippet\">...</td></tr>\
         <tr><td class=\"pesquisaMetatag\"><b>Unidade:</b> <a class=\"ancoraSigla\">U1</a></td></tr>"
    )
}

/// Número com `.` entre grupos de 3, como o total da barra observada.
pub(crate) fn milhar(n: u64) -> String {
    let digitos = n.to_string();
    let mut saida = String::new();
    for (i, c) in digitos.chars().enumerate() {
        if i > 0 && (digitos.len() - i).is_multiple_of(3) {
            saida.push('.');
        }
        saida.push(c);
    }
    saida
}

/// Barra, tabela com as linhas `inicio..=fim` e paginador `navegar()`.
pub(crate) fn conteudo_da_lista(inicio: u64, fim: u64, total: &str, tamanho: u64) -> String {
    let linhas: String = (inicio..=fim).map(linha).collect();
    conteudo_com_linhas(inicio, fim, total, tamanho, &linhas)
}

/// [`conteudo_da_lista`] com as linhas já montadas (processos ou documentos).
pub(crate) fn conteudo_com_linhas(
    inicio: u64,
    fim: u64,
    total: &str,
    tamanho: u64,
    linhas: &str,
) -> String {
    let numerico: u64 = total.replace('.', "").parse().unwrap_or(0);
    let mut links = String::new();
    for k in (0..10u64).map(|n| n * tamanho) {
        if k + 1 != inicio && k < numerico {
            let pagina = k / tamanho + 1;
            links.push_str(" <a href=\"javascript:navegar('");
            links.push_str(&k.to_string());
            links.push_str("')\">");
            links.push_str(&pagina.to_string());
            links.push_str("</a>\n");
        }
    }
    format!(
        "<div class=\"pesquisaBarra\"><div class=\"pesquisaBarraE\"><button type=\"button\" \
         id=\"btnVerCriteriosPesquisa\" class=\"infraButton\">Ver</button></div>\
         <div class=\"pesquisaBarraD\">Exibindo {} - {} de {total}</div></div>\
         <table border=\"0\" class=\"pesquisaResultado\">{linhas}</table>\
         <div class=\"pesquisaPaginas\"><div class=\"pesquisaPaginaSelecionada\">{}</div>\n{links}</div>",
        milhar(inicio),
        milhar(fim),
        (inicio - 1) / tamanho + 1
    )
}

/// Resposta 200 de uma lista, com o formulário ecoando o termo e `P` marcado.
pub(crate) fn resultado(termo: &str, inicio: u64, fim: u64, total: &str) -> String {
    pagina(&formulario(
        termo,
        true,
        &conteudo_da_lista(inicio, fim, total, 10),
    ))
}

/// Resposta 200 sem resultado, como na `UnB` (aviso dentro de `#conteudo`).
pub(crate) fn sem_resultado(termo: &str) -> String {
    pagina(&formulario(
        termo,
        true,
        "<div class=\"pesquisaSemResultado\">Nenhum resultado encontrado.<br/><br/>\
         Sugestões:<ul><li>Tente palavras-chave diferentes.</li></ul></div>",
    ))
}

/// Resposta 200 com a validação da Infra.
pub(crate) fn validacao(mensagem: &str) -> String {
    pagina(&format!(
        "<textarea id=\"txaInfraValidacao\" style=\"display:none\">{mensagem}</textarea>{}",
        formulario("", true, "")
    ))
}

/// `Location` do 302 para o processo único.
pub(crate) fn localizacao(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id}&id_documento=&{CONTEXTO}&infra_hash={}",
        hash('d')
    )
}

/// Lista que cabe numa página: a barra vem como `N resultados`, sem
/// "Exibindo" nem `navegar()` (`UnB` 4.0.9, 03/11/2026, com 2 e 9).
pub(crate) fn resultado_numa_pagina(termo: &str, linhas: u64, barra: &str) -> String {
    let lista = conteudo_da_lista(1, linhas, &linhas.to_string(), 10);
    let servida = format!("Exibindo 1 - {linhas} de {linhas}");
    assert!(lista.contains(&servida) && !lista.contains("navegar("));
    pagina(&formulario(termo, true, &lista.replace(&servida, barra)))
}
