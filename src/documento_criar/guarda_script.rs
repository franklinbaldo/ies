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
    ExpressaoRegular { dentro_de_classe: bool },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bloco {
    Outro,
    Submeter,
    ConfirmarDados,
    ConferirFormulario,
}

/// A forma servida precisa ser uma cadeia direta, sem ramos extras.
fn conferir_passos(
    linha: &str,
    blocos: &[Bloco],
    passo_submeter: &mut u8,
    passo_confirmar: &mut u8,
) -> Option<()> {
    match blocos {
        [Bloco::Submeter] if linha != "}" => {
            *passo_submeter = match (*passo_submeter, linha) {
                (0, DESABILITAR) => 1,
                (1, INSTRUCAO) => 2,
                (2, ENVIAR) => 3,
                _ => return None,
            };
        }
        [Bloco::ConfirmarDados] if linha != "}" => {
            // Captura Todos @323eb01: confirmacao so chama submeter()
            // depois de OnSubmitForm() aprovar o formulario.
            if *passo_confirmar != 0
                || (linha != "if (OnSubmitForm()) {" && linha != "if (OnSubmitForm()){")
            {
                return None;
            }
            *passo_confirmar = 1;
        }
        [Bloco::ConfirmarDados, Bloco::ConferirFormulario] if linha != "}" => {
            if *passo_confirmar != 1 || linha != "submeter();" {
                return None;
            }
            *passo_confirmar = 2;
        }
        _ if linha == INSTRUCAO => return None,
        _ => {}
    }
    Some(())
}

fn identificar_bloco(
    prefixo: &str,
    blocos: &[Bloco],
    funcoes_submeter: &mut usize,
    funcoes_confirmar: &mut usize,
) -> Option<Bloco> {
    if blocos.is_empty() && prefixo.trim() == "function submeter()" {
        *funcoes_submeter += 1;
        if *funcoes_submeter > 1 {
            return None;
        }
        Some(Bloco::Submeter)
    } else if blocos.is_empty() && prefixo.trim() == "function confirmarDados()" {
        *funcoes_confirmar += 1;
        if *funcoes_confirmar > 1 {
            return None;
        }
        Some(Bloco::ConfirmarDados)
    } else if blocos == [Bloco::ConfirmarDados] && prefixo.trim() == "if (OnSubmitForm())" {
        Some(Bloco::ConferirFormulario)
    } else {
        Some(Bloco::Outro)
    }
}

fn avancar_regex<I: Iterator<Item = char>>(
    caractere: char,
    restantes: &mut I,
    dentro_de_classe: bool,
) -> Estado {
    match caractere {
        '\\' => {
            restantes.next();
            Estado::ExpressaoRegular { dentro_de_classe }
        }
        '[' if !dentro_de_classe => Estado::ExpressaoRegular {
            dentro_de_classe: true,
        },
        ']' if dentro_de_classe => Estado::ExpressaoRegular {
            dentro_de_classe: false,
        },
        '/' if !dentro_de_classe => Estado::Codigo,
        _ => Estado::ExpressaoRegular { dentro_de_classe },
    }
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
            conferir_passos(linha, &blocos, &mut passo_submeter, &mut passo_confirmar)?;
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
                    // Um regexp literal em atribuicao pode conter chaves
                    // escapadas ou classes: nao sao blocos JS ativos.
                    '/' if prefixo.trim_end().ends_with('=') => {
                        estado = Estado::ExpressaoRegular {
                            dentro_de_classe: false,
                        };
                    }
                    '{' => blocos.push(identificar_bloco(
                        &prefixo,
                        &blocos,
                        &mut funcoes_submeter,
                        &mut funcoes_confirmar,
                    )?),
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
                Estado::ExpressaoRegular { dentro_de_classe } => {
                    estado = avancar_regex(c, &mut caracteres, dentro_de_classe);
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

/// A arvore do parser inclui nos de `template`, mas o navegador
/// conserva seu conteudo em `DocumentFragment` inerte. A posicao de cada
/// controle deve estar fora dessas faixas para constituir evidencia.
pub(super) fn faixas_inertes(dom: &tl::VDom<'_>) -> Option<Vec<(usize, usize)>> {
    let mut faixas = Vec::new();
    for no in dom.query_selector("template").into_iter().flatten() {
        let tag = no.get(dom.parser()).and_then(tl::Node::as_tag)?;
        faixas.push(tag.boundaries(dom.parser()));
    }
    Some(faixas)
}

pub(super) fn dentro_de_template(inicio: usize, faixas: &[(usize, usize)]) -> bool {
    faixas
        .iter()
        .any(|(primeiro, ultimo)| inicio > *primeiro && inicio <= *ultimo)
}

/// Elementos com atributo HTML `hidden` (no próprio controle ou em
/// ancestral) não oferecem um acionador selecionável.
pub(super) fn faixas_ocultas(dom: &tl::VDom<'_>) -> Option<Vec<(usize, usize)>> {
    let mut faixas = Vec::new();
    for no in dom.nodes() {
        let Some(tag) = no.as_tag() else {
            continue;
        };
        if crate::html_atributos::atributo_presente(tag, "hidden").ok()? {
            faixas.push(tag.boundaries(dom.parser()));
        }
    }
    Some(faixas)
}

pub(super) fn dentro_de_faixa_oculta(inicio: usize, faixas: &[(usize, usize)]) -> bool {
    faixas
        .iter()
        .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio))
}

/// Controles do cadastro também são inativos sob fieldset disabled.
/// Não estender essa regra aos links da seleção: anchors não são
/// automaticamente desabilitados por fieldset disabled.
pub(super) fn faixas_de_controles_inativos(dom: &tl::VDom<'_>) -> Option<Vec<(usize, usize)>> {
    let mut faixas = faixas_ocultas(dom)?;
    for no in dom.query_selector("fieldset").into_iter().flatten() {
        let tag = no.get(dom.parser()).and_then(tl::Node::as_tag)?;
        if crate::html_atributos::atributo_presente(tag, "disabled").ok()? {
            faixas.push(tag.boundaries(dom.parser()));
        }
    }
    Some(faixas)
}

/// A captura 4.0.9 publica btnSalvar como button habilitado com
/// `onclick=confirmarDados()`; a mera funcao `submeter()` morta nao prova
/// que a tela oferece a escrita. Topo e rodape podem repetir o botao.
fn botoes_salvar_publicam_acao(dom: &tl::VDom<'_>, faixas: &[(usize, usize)]) -> bool {
    let Some(faixas_inativas) = faixas_de_controles_inativos(dom) else {
        return false;
    };
    let mut encontrados = 0_usize;
    for no in dom.query_selector("button").into_iter().flatten() {
        let Some(botao) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            return false;
        };
        if dentro_de_template(botao.boundaries(dom.parser()).0, faixas) {
            continue;
        }
        let Ok(id) = crate::html_atributos::valor_do_atributo(botao, "id") else {
            return false;
        };
        if id.as_deref() != Some("btnSalvar") {
            continue;
        }
        let inicio = botao.boundaries(dom.parser()).0;
        if faixas_inativas
            .iter()
            .any(|&(primeiro, ultimo)| (primeiro..=ultimo).contains(&inicio))
        {
            return false;
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

/// Extrai a fonte literal do elemento `script` no HTML recebido.
/// O DOM identifica o elemento ativo e valida seus atributos; `inner_text`
/// perde nós que o parser tl tratou como markup dentro de strings JS.
/// Encerrar no primeiro `</script>` é a fronteira HTML: mesmo dentro de
/// uma string JS, a página Web encerra ali o elemento.
pub(super) fn fonte_do_script(html: &str, inicio: usize) -> Option<&str> {
    let bytes = html.as_bytes();
    if !bytes
        .get(inicio..inicio + 7)?
        .eq_ignore_ascii_case(b"<script")
    {
        return None;
    }
    let mut i = inicio + 7;
    if !matches!(
        bytes.get(i).copied(),
        Some(b'>' | b' ' | b'\t' | b'\r' | b'\n')
    ) {
        return None;
    }
    let mut aspas = None;
    loop {
        let atual = *bytes.get(i)?;
        match (aspas, atual) {
            (None, b'\'' | b'"') => aspas = Some(atual),
            (Some(aberta), c) if c == aberta => aspas = None,
            (None, b'>') => {
                i += 1;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let inicio_corpo = i;
    while i < bytes.len() {
        if bytes
            .get(i..i + 8)
            .is_some_and(|trecho| trecho.eq_ignore_ascii_case(b"</script"))
        {
            let mut fim = i + 8;
            let proximo = *bytes.get(fim)?;
            if proximo == b'>' || proximo.is_ascii_whitespace() {
                while bytes.get(fim).is_some_and(u8::is_ascii_whitespace) {
                    fim += 1;
                }
                // Fechamento malformado e ambiguo nao autoriza escrita.
                if bytes.get(fim).copied() != Some(b'>') {
                    return None;
                }
                return html.get(inicio_corpo..i);
            }
        }
        i += 1;
    }
    None
}

/// Somente script inline clássico, sem src. Um segundo corpo que
/// redefina `submeter()` torna a prova ambígua. Falha fechada antes do POST.
pub(super) fn cadastro_publica_script_de_criacao(html: &str) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let Some(faixas) = faixas_inertes(&dom) else {
        return false;
    };
    if !botoes_salvar_publicam_acao(&dom, &faixas) {
        return false;
    }
    let mut ocorrencias = 0;
    let mut funcoes_submeter = 0;
    let mut confirmacoes = 0;
    for no in dom.query_selector("script").into_iter().flatten() {
        let Some(script) = no.get(dom.parser()).and_then(tl::Node::as_tag) else {
            return false;
        };
        if dentro_de_template(script.boundaries(dom.parser()).0, &faixas) {
            continue;
        }
        let Ok(tipo) = crate::html_atributos::valor_do_atributo(script, "type") else {
            return false;
        };
        // application/javascript tambem executa. Um script module ou
        // externo poderia substituir o callback sem que seu corpo fosse
        // comprovado: nao autorizar a escrita nessa forma.
        let tipo = tipo.as_deref().map(str::trim).unwrap_or("");
        if tipo.eq_ignore_ascii_case("module") {
            return false;
        }
        if !tipo.is_empty()
            && ![
                "text/javascript",
                "application/javascript",
                "text/ecmascript",
                "application/ecmascript",
            ]
            .iter()
            .any(|mime| tipo.eq_ignore_ascii_case(mime))
        {
            // JSON, importmap e tipos de dados nao sao codigo executado.
            continue;
        }
        match crate::html_atributos::atributo_presente(script, "src") {
            Ok(true) | Err(_) => return false,
            Ok(false) => {}
        }
        // Usar a fonte original em vez de inner_text/inner_html do DOM:
        // a serializacao pode perder markup de literais JS. O inicio e
        // vinculado ao script ja validado (tipo/src/template conferidos).
        let Some(texto) = fonte_do_script(html, script.boundaries(dom.parser()).0) else {
            return false;
        };
        let Some((instrucoes, funcoes, chamadas)) = contar_instrucoes(texto) else {
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
