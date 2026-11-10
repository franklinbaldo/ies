//! Identificação conservadora de JavaScript estático da ciência documental.
//! Sem execução de JS, sem regex e sem interpretar ramos incertos.

use super::ErroCienciaNaArvore;

/// Identifica somente código ativo; comentários, literais e templates
/// não podem autorizar um GET mutador. Retorna a profundidade dos blocos.
pub(crate) fn niveis_js(script: &str) -> Option<Vec<Option<usize>>> {
    let bytes = script.as_bytes();
    let mut niveis = vec![None; bytes.len()];
    let (mut i, mut profundidade) = (0, 0usize);
    while i < bytes.len() {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i += 2;
            let mut fechado = false;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    fechado = true;
                    break;
                }
                i += 1;
            }
            if !fechado {
                return None;
            }
            continue;
        }
        if matches!(bytes[i], b'\'' | b'"' | b'\x60') {
            let aspas = bytes[i];
            i += 1;
            let mut fechado = false;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == aspas {
                    i += 1;
                    fechado = true;
                    break;
                }
                if bytes[i] == b'\n' && aspas != b'\x60' {
                    return None;
                }
                i += 1;
            }
            if !fechado {
                return None;
            }
            continue;
        }
        match bytes[i] {
            b'{' => {
                niveis[i] = Some(profundidade);
                profundidade += 1;
            }
            b'}' => {
                profundidade = profundidade.checked_sub(1)?;
                niveis[i] = Some(profundidade);
            }
            _ => niveis[i] = Some(profundidade),
        }
        i += 1;
    }
    (profundidade == 0).then_some(niveis)
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
