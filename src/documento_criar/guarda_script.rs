//! Guarda conservadora da criação de documento (#246).
//! Reconhece a forma literal observada no SEI; não executa JavaScript.
//! Ramo inatingível ou script não comprovado nunca autorizam POST.

const INSTRUCAO: &str = "document.getElementById('hdnFlagDocumentoCadastro').value = '2';";
const DESABILITAR: &str = "desabilitarBotaoSalvar(true);";
const ENVIAR: &str = "document.getElementById('frmDocumentoCadastro').submit();";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Estado {
    Codigo,
    ComentarioDeBloco,
    Cadeia(char),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bloco {
    Outro,
    Submeter,
    ConfirmarDados,
    ConferirFormulario,
}

/// Ler as linhas como texto, excluindo comentários, strings e blocos
/// aninhados. Dentro de `submeter()` só vale a sequência da captura
/// Todos `tests/paginas_reais/documento_gerar.html` @323eb01.
fn contar_instrucoes(texto: &str) -> Option<(usize, usize, usize)> {
    let mut ocorrencias = 0;
    let mut funcoes_submeter = 0;
    let mut confirmacoes = 0;
    let mut funcoes_confirmar = 0;
    let mut estado = Estado::Codigo;
    let mut blocos: Vec<Bloco> = Vec::new();
    let mut passo_submeter = 0_u8;
    let mut passo_confirmar = 0_u8;
    for linha in texto.lines() {
        let linha = linha.trim();
        if estado == Estado::Codigo {
            match blocos.as_slice() {
                [Bloco::Submeter] if linha != "}" => {
                    passo_submeter = match (passo_submeter, linha) {
                        (0, DESABILITAR) => 1,
                        (1, INSTRUCAO) => 2,
                        (2, ENVIAR) => 3,
                        _ => return None,
                    };
                }
                [Bloco::ConfirmarDados] if linha != "}" => {
                    // Captura Todos @323eb01: a confirmação só chama
                    // submeter() se a validação do formulário passar.
                    if passo_confirmar != 0
                        || (linha != "if (OnSubmitForm()) {"
                            && linha != "if (OnSubmitForm()){")
                    {
                        return None;
                    }
                    passo_confirmar = 1;
                }
                [Bloco::ConfirmarDados, Bloco::ConferirFormulario] if linha != "}" => {
                    if passo_confirmar != 1 || linha != "submeter();" {
                        return None;
                    }
                    passo_confirmar = 2;
                }
                _ if linha == INSTRUCAO => {
                    // A flag isolada não demonstra uma escrita acionável.
                    return None;
                }
                _ => {}
            }
        }
        let mut prefixo = String::new();
        let mut caracteres = linha.chars().peekable();
        while let Some(c) = caracteres.next() {
            match estado {
                Estado::Codigo => match c {
                    '/' if caracteres.peek().copied() == Some('/') => break,
                    '/' if caracteres.peek().copied() == Some('*') => {
                        caracteres.next();
                        estado = Estado::ComentarioDeBloco;
                    }
                    '\'' | '"' | '`' => estado = Estado::Cadeia(c),
                    '{' => {
                        let bloco = if blocos.is_empty() && prefixo.trim() == "function submeter()" {
                            funcoes_submeter += 1;
                            if funcoes_submeter > 1 {
                                return None;
                            }
                            Bloco::Submeter
                        } else if blocos.is_empty()
                            && prefixo.trim() == "function confirmarDados()"
                        {
                            funcoes_confirmar += 1;
                            if funcoes_confirmar > 1 {
                                return None;
                            }
                            Bloco::ConfirmarDados
                        } else if blocos.as_slice() == [Bloco::ConfirmarDados]
                            && prefixo.trim() == "if (OnSubmitForm())"
                        {
                            Bloco::ConferirFormulario
                        } else {
                            Bloco::Outro
                        };
                        blocos.push(bloco);
                    }
                    '}' => match blocos.pop()? {
                        Bloco::Submeter => {
                            if passo_submeter != 3 {
                                return None;
                            }
                            ocorrencias += 1;
                            passo_submeter = 0;
                        }
                        Bloco::ConferirFormulario => {
                            if passo_confirmar != 2 {
                                return None;
                            }
                            passo_confirmar = 3;
                        }
                        Bloco::ConfirmarDados => {
                            if passo_confirmar != 3 {
                                return None;
                            }
                            confirmacoes += 1;
                            passo_confirmar = 0;
                        }
                        Bloco::Outro => {}
                    },
                    _ => prefixo.push(c),
                },
                Estado::ComentarioDeBloco => {
                    if c == '*' && caracteres.peek().copied() == Some('/') {
                        caracteres.next();
                        estado = Estado::Codigo;
                    }
                }
                Estado::Cadeia(delimitador) => {
                    if c == '\\' {
                        caracteres.next();
                    } else if c == delimitador {
                        estado = Estado::Codigo;
                    }
                }
            }
        }
        if ocorrencias > 1 {
            return None;
        }
    }
    // Literais, comentários ou blocos incompletos não demonstram nada.
    if estado != Estado::Codigo || !blocos.is_empty() {
        return None;
    }
    Some((ocorrencias, funcoes_submeter, confirmacoes))
}

/// A captura 4.0.9 publica btnSalvar como button habilitado com
/// `onclick=confirmarDados()`; a mera funcao `submeter()` morta nao prova
/// que a tela oferece a escrita. Topo e rodape podem repetir o botao.
fn botoes_salvar_publicam_acao(dom: &tl::VDom<'_>) -> bool {
    let mut encontrados = 0_usize;
    for no in dom.query_selector("button").into_iter().flatten() {
        let Some(botao) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            return false;
        };
        let Ok(id) = crate::html_atributos::valor_do_atributo(botao, "id") else {
            return false;
        };
        if id.as_deref() != Some("btnSalvar") {
            continue;
        }
        encontrados += 1;
        let Ok(tipo) = crate::html_atributos::valor_do_atributo(botao, "type") else {
            return false;
        };
        let Ok(nome) = crate::html_atributos::valor_do_atributo(botao, "name") else {
            return false;
        };
        let Ok(acionador) = crate::html_atributos::valor_do_atributo(botao, "onclick") else {
            return false;
        };
        let Ok(desabilitado) = crate::html_atributos::atributo_presente(botao, "disabled") else {
            return false;
        };
        if desabilitado
            || tipo.as_deref() != Some("button")
            || nome.as_deref() != Some("btnSalvar")
            || acionador.as_deref().map(str::trim) != Some("confirmarDados()")
        {
            return false;
        }
    }
    encontrados > 0
}

/// Somente script inline clássico, sem src. Um segundo corpo que
/// redefina `submeter()` torna a prova ambígua. Falha fechada antes do POST.
pub(super) fn cadastro_publica_script_de_criacao(html: &str) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    if !botoes_salvar_publicam_acao(&dom) {
        return false;
    }
    let mut ocorrencias = 0;
    let mut funcoes_submeter = 0;
    let mut confirmacoes = 0;
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(script) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            return false;
        };
        let Ok(tipo) = crate::html_atributos::valor_do_atributo(script, "type") else {
            return false;
        };
        if tipo
            .as_deref()
            .is_some_and(|tipo| !tipo.trim().eq_ignore_ascii_case("text/javascript"))
        {
            continue;
        }
        match crate::html_atributos::atributo_presente(script, "src") {
            Ok(true) => continue,
            Ok(false) => {}
            Err(_) => return false,
        }
        let texto = script.inner_text(dom.parser());
        let Some((instrucoes, funcoes, chamadas)) = contar_instrucoes(&texto) else {
            return false;
        };
        ocorrencias += instrucoes;
        funcoes_submeter += funcoes;
        confirmacoes += chamadas;
        if ocorrencias > 1 || funcoes_submeter > 1 || confirmacoes > 1 {
            return false;
        }
    }
    ocorrencias == 1 && funcoes_submeter == 1 && confirmacoes == 1
}
