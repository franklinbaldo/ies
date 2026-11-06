//! Validação pura da oferta de exclusão do Acompanhamento Especial (#254).
//! A função `acaoExcluir` é examinada como texto; nunca é executada.
//! Estrutura derivada da `UnB` em Todos `tests/test_processo_acompanhar_010.py` @323eb01.

use super::{ErroAcompanhar, TABELA, campo};
use crate::erro_html::ErroHtml;
use crate::html_atributos::atributo_presente;
use crate::instalacao::Instalacao;

const FORMULARIO: &str = "frmGerenciarAcompanhamento";
const OCULTO: &str = "hdnInfraItemId";

/// Comprova que a linha escolhida oferece exclusão pelo formulário servido.
/// Não emite requisição. Devolve SOMENTE o destino assinado extraído
/// do script servido e conferido para este processo; nunca executa JS.
pub(super) fn conferir_oferta_exclusao(
    instalacao: &Instalacao,
    html: &str,
    id_procedimento: &str,
    id_alvo: &str,
) -> Result<String, ErroAcompanhar> {
    use ErroAcompanhar as E;
    let sem_oferta = E::ExclusaoNaoOferecida;
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(ErroHtml::html("html_de_acompanhamento_invalido").into());
    }
    if id_alvo.is_empty() || !id_alvo.chars().all(|c| c.is_ascii_digit()) {
        return Err(sem_oferta);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ErroHtml::html("html_de_acompanhamento_invalido"))?;
    let parser = dom.parser();
    let mut form = None;
    for no in dom.query_selector("form").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
        if campo(tag, "id")?.as_deref() == Some(FORMULARIO) && form.replace(tag).is_some() {
            return Err(sem_oferta);
        }
    }
    let form = form.ok_or(sem_oferta)?;
    // getElementById consulta TODO o DOM. Um elemento fora do form
    // com o mesmo ID pode receber o valor em vez do oculto enviável.
    let mut ids_do_oculto = 0;
    for no in dom.query_selector("[id]").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
        if campo(tag, "id")?.as_deref() == Some(OCULTO) {
            ids_do_oculto += 1;
        }
    }
    if ids_do_oculto != 1 {
        return Err(sem_oferta);
    }
    if campo(form, "method")?.as_deref() != Some("post") {
        return Err(sem_oferta);
    }
    let action = campo(form, "action")?.ok_or(sem_oferta)?;
    let action = crate::formulario_web::decodificar_atributo(&action).ok_or(sem_oferta)?;
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &action,
        "acompanhamento_gerenciar",
        id_procedimento,
    )
    .map_err(|_| sem_oferta)?;

    let mut ocultos = 0;
    let mut tabelas = Vec::new();
    for no in form.children().all(parser) {
        let Some(tag) = no.as_tag() else {
            continue;
        };
        if tag.name() == "input" && campo(tag, "name")?.as_deref() == Some(OCULTO) {
            if campo(tag, "type")?.as_deref() != Some("hidden")
                || campo(tag, "id")?.as_deref() != Some(OCULTO)
                || campo(tag, "value")?.as_deref() != Some("")
                || atributo_presente(tag, "disabled").map_err(|e| ErroHtml::atributo("html_de_acompanhamento_invalido", e))?
            {
                return Err(sem_oferta);
            }
            ocultos += 1;
        }
        if tag.name() == "table" && campo(tag, "summary")?.as_deref() == Some(TABELA) {
            tabelas.push(tag);
        }
    }
    if ocultos != 1 || tabelas.len() != 1 {
        return Err(sem_oferta);
    }
    conferir_linha(tabelas[0], parser, id_alvo)?;
    conferir_script(instalacao, &dom, id_procedimento)
}

fn conferir_linha(
    tabela: &tl::HTMLTag<'_>,
    parser: &tl::Parser<'_>,
    id_alvo: &str,
) -> Result<(), ErroAcompanhar> {
    let sem_oferta = ErroAcompanhar::ExclusaoNaoOferecida;
    let prefixo = format!("acaoExcluir('{id_alvo}',");
    let mut linhas_do_alvo = 0;
    for no in tabela.children().all(parser) {
        let Some(tr) = no.as_tag() else {
            continue;
        };
        if tr.name() != "tr" {
            continue;
        }
        let mut alvo = false;
        let mut links = 0;
        for descendente in tr.children().all(parser) {
            let Some(tag) = descendente.as_tag() else {
                continue;
            };
            if tag.name() == "input"
                && campo(tag, "type")?.as_deref() == Some("checkbox")
                && campo(tag, "value")?.as_deref() == Some(id_alvo)
            {
                alvo = true;
            }
            if tag.name() == "a"
                && let Some(onclick) = campo(tag, "onclick")?
            {
                let onclick =
                    crate::formulario_web::decodificar_atributo(&onclick).ok_or(sem_oferta)?;
                let onclick = onclick.trim();
                if onclick.starts_with(&prefixo) && onclick.ends_with(");") {
                    links += 1;
                }
            }
        }
        if alvo {
            linhas_do_alvo += 1;
            if links != 1 {
                return Err(sem_oferta);
            }
        }
    }
    if linhas_do_alvo != 1 {
        return Err(sem_oferta);
    }

    Ok(())
}

/// Localiza a declaração global exigida pelo onclick, ignorando texto de
/// comentários e strings. Não interpreta JS: blocos, expressões de função,
/// templates e sintaxe não reconhecida não provam uma função global.
/// Os bytes ASCII de controle não são parte de sequências UTF-8 multibyte.
fn inicio_declaracao_global(codigo: &str) -> Option<usize> {
    let assinatura = b"function acaoExcluir(";
    let bytes = codigo.as_bytes();
    let mut i = 0;
    let mut nivel = 0usize;
    let mut aspas: Option<u8> = None;
    let mut escapado = false;
    let mut comentario_linha = false;
    let mut comentario_bloco = false;
    let mut anterior = None;

    while i < bytes.len() {
        let c = bytes[i];
        let proximo = bytes.get(i + 1).copied();
        if comentario_linha {
            if c == 10 || c == 13 {
                comentario_linha = false;
            }
            i += 1;
            continue;
        }
        if comentario_bloco {
            if c == b'*' && proximo == Some(b'/') {
                comentario_bloco = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if let Some(delimitador) = aspas {
            if escapado {
                escapado = false;
            } else if c == 92 {
                escapado = true;
            } else if c == delimitador {
                aspas = None;
            }
            i += 1;
            continue;
        }
        if c == b'/' && proximo == Some(b'/') {
            comentario_linha = true;
            i += 2;
            continue;
        }
        if c == b'/' && proximo == Some(b'*') {
            comentario_bloco = true;
            i += 2;
            continue;
        }
        if c == 39 || c == 34 {
            aspas = Some(c);
            i += 1;
            continue;
        }
        // Backticks (96) e barra não comentada podem ter semântica
        // de template/regex: sem prova segura, recusar conservadoramente.
        if c == 96 || c == b'/' {
            return None;
        }
        if nivel == 0 && bytes[i..].starts_with(assinatura) {
            // Declaração em posição de comando global, e não expressão
            // "const x = function acaoExcluir(...)" ou trecho em string.
            if matches!(anterior, None | Some(b';' | b'}')) {
                return Some(i + assinatura.len());
            }
            return None;
        }
        match c {
            b'{' => nivel += 1,
            b'}' => nivel = nivel.checked_sub(1)?,
            _ => {}
        }
        if nivel == 0 && !c.is_ascii_whitespace() {
            anterior = Some(c);
        }
        i += 1;
    }
    None
}

/// Identifica o corpo da única função observada, sem avaliar JavaScript.
/// Chaves em strings não delimitam blocos; comentários e template literals
/// não são suportados: qualquer variante ambígua falha fechada.
fn corpo_da_acao_excluir(codigo: &str) -> Option<&str> {
    let inicio = inicio_declaracao_global(codigo)?;
    let (parametros, resto) = codigo[inicio..].split_once(')')?;
    let mut nomes = parametros.split(',').map(str::trim);
    if nomes.next() != Some("id")
        || nomes.next().is_none_or(str::is_empty)
        || nomes.next().is_some()
    {
        return None;
    }
    let inicio_corpo = resto.trim_start().strip_prefix('{')?;
    let mut nivel = 1usize;
    let mut aspas = None;
    let mut escape = false;
    for (i, c) in inicio_corpo.char_indices() {
        if let Some(delimitador) = aspas {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == delimitador {
                aspas = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => aspas = Some(c),
            '`' => return None,
            '{' => nivel += 1,
            '}' => {
                nivel -= 1;
                if nivel == 0 {
                    let corpo = &inicio_corpo[..i];
                    if corpo.contains("//") || corpo.contains("/*") {
                        return None;
                    }
                    return Some(corpo);
                }
            }
            _ => {}
        }
    }
    None
}

fn conferir_script(
    instalacao: &Instalacao,
    dom: &tl::VDom<'_>,
    id_procedimento: &str,
) -> Result<String, ErroAcompanhar> {
    use ErroAcompanhar as E;
    let sem_oferta = E::ExclusaoNaoOferecida;
    let parser = dom.parser();
    let mut scripts = Vec::new();
    for no in dom.query_selector("script").into_iter().flatten() {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_de_acompanhamento_invalido"))?;
        let codigo = tag.inner_text(parser);
        if codigo.contains("function acaoExcluir(") {
            // Texto em <script type="application/json">, escopo module ou
            // <script src> não publica a função global que o onclick chama.
            // Nunca tomar código inerte como prova de oferta de escrita.
            let tipo = campo(tag, "type")?;
            let tipo_classico = tipo.as_deref().is_none_or(|valor| {
                let valor = valor.trim();
                valor.eq_ignore_ascii_case("text/javascript")
                    || valor.eq_ignore_ascii_case("application/javascript")
                    || valor.is_empty()
            });
            if !tipo_classico
                || atributo_presente(tag, "src").map_err(|e| ErroHtml::atributo("html_de_acompanhamento_invalido", e))?
            {
                return Err(sem_oferta);
            }
            scripts.push(codigo.to_string());
        }
    }
    let codigo = match scripts.as_slice() {
        [codigo] if codigo.matches("function acaoExcluir(").count() == 1 => codigo,
        _ => return Err(sem_oferta),
    };
    // Não confundir trechos em outra função (ou fora de qualquer função)
    // com o que acaoExcluir realmente executaria. A extração não avalia JS:
    // reconhece só o cabeçalho observado e as chaves balanceadas.
    let corpo = corpo_da_acao_excluir(codigo).ok_or(sem_oferta)?;
    let gravar = "document.getElementById('hdnInfraItemId').value=id;";
    let atribuir = "document.getElementById('frmGerenciarAcompanhamento').action='";
    let enviar = "document.getElementById('frmGerenciarAcompanhamento').submit();";
    if corpo.matches(gravar).count() != 1
        || corpo.matches(atribuir).count() != 1
        || corpo.matches(enviar).count() != 1
    {
        return Err(sem_oferta);
    }
    let i_gravar = corpo.find(gravar).ok_or(sem_oferta)?;
    let i_atribuir = corpo.find(atribuir).ok_or(sem_oferta)?;
    let i_enviar = corpo.find(enviar).ok_or(sem_oferta)?;
    if !(i_gravar < i_atribuir && i_atribuir < i_enviar) {
        return Err(sem_oferta);
    }
    let depois_do_atribuir = &corpo[i_atribuir + atribuir.len()..];
    let (destino, _) = depois_do_atribuir.split_once('\'').ok_or(sem_oferta)?;
    // Somente a forma observada na UnB: uma confirmação com três
    // instruções, sem instrução morta, retorno antecipado ou código extra.
    let inicio: String = corpo[..i_gravar]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let condicao_observada = (inicio.starts_with("if(confirm(\"") && inicio.ends_with("\")){"))
        || (inicio.starts_with("if(confirm('") && inicio.ends_with("')){"));
    let entre_gravar_e_atribuir = corpo[i_gravar + gravar.len()..i_atribuir].trim();
    let depois_do_destino = i_atribuir + atribuir.len() + destino.len() + 1;
    if !condicao_observada
        || !entre_gravar_e_atribuir.is_empty()
        || corpo[depois_do_destino..i_enviar].trim() != ";"
        || corpo[i_enviar + enviar.len()..].trim() != "}"
    {
        return Err(sem_oferta);
    }
    let destino = crate::formulario_web::decodificar_atributo(destino).ok_or(sem_oferta)?;
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &destino,
        "acompanhamento_excluir",
        id_procedimento,
    )
    .map_err(|_| sem_oferta)
}

#[cfg(test)]
#[path = "tests_exclusao.rs"]
mod tests;
