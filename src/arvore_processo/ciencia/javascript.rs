//! Identificação conservadora de JavaScript estático da ciência documental.
//! Sem execução de JS, sem regex e sem interpretar ramos incertos.

use super::ErroCienciaNaArvore;

/// Identifica somente código ativo; comentários, literais e templates
/// não podem autorizar um GET mutador. Retorna a profundidade dos blocos.
pub(crate) fn niveis_js(script: &str) -> Option<Vec<Option<usize>>> {
    let bytes = script.as_bytes();
    let mut niveis = vec![None; bytes.len()];
    let (mut i, mut profundidade) = (0, 0usize);
    while let Some(&byte) = bytes.get(i) {
        let seguinte = bytes.get(i + 1).copied();
        if byte == b'/' && seguinte == Some(b'/') {
            i += 2;
            while bytes.get(i).is_some_and(|&atual| atual != b'\n') {
                i += 1;
            }
            continue;
        }
        if byte == b'/' && seguinte == Some(b'*') {
            i = fim_do_comentario_de_bloco(bytes, i + 2)?;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'\x60') {
            i = fim_do_literal(bytes, i + 1, byte)?;
            continue;
        }
        if byte == b'}' {
            profundidade = profundidade.checked_sub(1)?;
        }
        *niveis.get_mut(i)? = Some(profundidade);
        if byte == b'{' {
            profundidade += 1;
        }
        i += 1;
    }
    (profundidade == 0).then_some(niveis)
}

/// Posição depois do `*/`; `None` se o comentário não fecha.
fn fim_do_comentario_de_bloco(bytes: &[u8], mut i: usize) -> Option<usize> {
    while let (Some(&atual), Some(&depois)) = (bytes.get(i), bytes.get(i + 1)) {
        if atual == b'*' && depois == b'/' {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

/// Posição depois da aspa de fechamento; `None` se o literal não fecha ou
/// se uma aspa simples ou dupla atravessa a linha.
fn fim_do_literal(bytes: &[u8], mut i: usize, aspas: u8) -> Option<usize> {
    while let Some(&atual) = bytes.get(i) {
        if atual == b'\\' {
            i += 2;
            continue;
        }
        if atual == aspas {
            return Some(i + 1);
        }
        if atual == b'\n' && aspas != b'\x60' {
            return None;
        }
        i += 1;
    }
    None
}

/// Aceita só as duas formas estáticas observadas, fora de comentários,
/// templates e condicionais: variável no topo ou função SEI 4.0.
pub(super) fn urls_ativas_da_ciencia_do_documento(
    script: &str,
) -> Result<Vec<String>, ErroCienciaNaArvore> {
    let niveis =
        niveis_js(script).ok_or(ErroCienciaNaArvore::ScriptDaCienciaDoDocumentoForaDaForma)?;
    let mut linhas = Vec::new();
    let mut inicio = 0;
    for linha in script.split_inclusive('\n') {
        let limpa = linha.trim();
        let posicao = linha.find(limpa).unwrap_or(0);
        let nivel = niveis.get(inicio + posicao).copied().flatten();
        linhas.push((limpa, nivel));
        inicio += linha.len();
    }
    let mut urls = Vec::new();
    let mut condicionada = false;
    for (i, &(linha, nivel)) in linhas.iter().enumerate() {
        if nivel != Some(0) {
            continue;
        }
        // Uma condicional no topo (mesmo sem chaves) impede concluir
        // estaticamente que uma declaração posterior esteja ativa.
        if [
            "if(", "if ", "else", "switch", "for(", "for ", "while", "with", "try", "catch",
        ]
        .iter()
        .any(|prefixo| linha.starts_with(prefixo))
        {
            condicionada = true;
            continue;
        }
        if condicionada {
            continue;
        }
        if let Some(resto) = linha.strip_prefix("var linkCienciaDocumento = '") {
            let url = resto
                .strip_suffix("';")
                .filter(|url| !url.contains('\\') && !url.contains('\''))
                .ok_or(ErroCienciaNaArvore::LinkDeCienciaDoDocumentoForaDaForma)?;
            urls.push(url.to_owned());
        }
        if linha == "function cienciaDocumento(){" {
            let confirmacao = linhas.get(i + 1).map_or("", |l| l.0);
            let (destino, nivel_destino) = linhas.get(i + 2).copied().unwrap_or(("", None));
            if !confirmacao.starts_with("//if (confirm(")
                || !confirmacao.ends_with("){")
                || nivel_destino != Some(1)
            {
                return Err(ErroCienciaNaArvore::FuncaoDeCienciaDoDocumentoForaDaForma);
            }
            let url = destino
                .strip_prefix("location.href = '")
                .and_then(|resto| resto.strip_suffix("';"))
                .filter(|url| !url.contains('\\') && !url.contains('\''))
                .ok_or(ErroCienciaNaArvore::FuncaoDeCienciaDoDocumentoForaDaForma)?;
            urls.push(url.to_owned());
        }
    }
    Ok(urls)
}
