//! Guarda conservadora da criação de documento (#246).
//! Extrai controles do DOM para formar uma submissão HTTP sem executar JS.
//! Aceite exige resposta causal do servidor e releitura independente.

/// A captura 4.0.9 publica btnSalvar como button habilitado com
/// `onclick=confirmarDados()`; a mera funcao `submeter()` morta nao prova
/// que a tela oferece a escrita. Topo e rodape podem repetir o botao.
/// Um button type=button nao participa do corpo do submit: seu atributo
/// name e opcional, desde que id, tipo, onclick e disponibilidade coincidam.
fn botoes_salvar_publicam_acao(dom: &crate::tela_html::Documento) -> bool {
    let mut encontrados = 0_usize;
    for botao in dom.elementos("button") {
        if botao.atributo("id") != Some("btnSalvar") {
            continue;
        }
        encontrados += 1;
        if botao.presente("disabled")
            || botao.atributo("type") != Some("button")
            || botao.atributo("onclick").map(str::trim) != Some("confirmarDados()")
        {
            return false;
        }
    }
    encontrados > 0
}

/// Verifica somente os controles oferecidos pelo cadastro observado.
/// A submissão HTTP é derivada do formulário servido e seu aceite exige
/// 302 causal e releitura independente. Interpretar JavaScript para tentar
/// simular a cadeia dos callbacks não é parte desse contrato.
pub(super) fn cadastro_oferece_salvar(html: &crate::tela_html::HtmlServido) -> bool {
    botoes_salvar_publicam_acao(&crate::tela_html::arvore(html))
}
