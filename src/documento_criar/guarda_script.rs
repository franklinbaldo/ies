//! Guarda conservadora da criação de documento (#246).
//! Extrai controles do DOM para formar uma submissão HTTP sem executar JS.
//! Aceite exige resposta causal do servidor e releitura independente.

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
/// Um button type=button nao participa do corpo do submit: seu atributo
/// name e opcional, desde que id, tipo, onclick e disponibilidade coincidam.
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
        let Ok(acionador) = crate::html_atributos::valor_do_atributo(botao, "onclick") else {
            return false;
        };
        let Ok(desabilitado) = crate::html_atributos::atributo_presente(botao, "disabled") else {
            return false;
        };
        if desabilitado
            || tipo.as_deref() != Some("button")
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

/// Verifica somente os controles oferecidos pelo cadastro observado.
/// A submissão HTTP é derivada do formulário servido e seu aceite exige
/// 302 causal e releitura independente. Interpretar JavaScript para tentar
/// simular a cadeia dos callbacks não é parte desse contrato.
pub(super) fn cadastro_oferece_salvar(html: &crate::tela_html::HtmlServido) -> bool {
    let dom = crate::tela_html::interpretar(html);
    let Some(faixas) = faixas_inertes(&dom) else {
        return false;
    };
    botoes_salvar_publicam_acao(&dom, &faixas)
}
