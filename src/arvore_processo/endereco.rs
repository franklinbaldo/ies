//! Conferência comum dos endereços assinados que a tela do processo e a
//! árvore publicam: `controlador.php?…` relativo (a forma servida), com uma
//! `acao` igual à da operação, um `id_procedimento` igual ao aberto e um
//! `infra_hash` de 64 hex, chaves canônicas. Resolvido contra a instalação;
//! sem regex.
use crate::instalacao::Instalacao;
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnderecoRecusado {
    NaoReconhecido,
    DeOutroProcesso,
}

/// Devolve a URL absoluta na instalação.
pub(crate) fn conferir_endereco_relativo(
    instalacao: &Instalacao,
    src: &str,
    acao: &str,
    id: &str,
) -> Result<String, EnderecoRecusado> {
    use EnderecoRecusado as E;
    let consulta = src
        .strip_prefix("controlador.php?")
        .ok_or(E::NaoReconhecido)?;
    if src.len() > 8_192
        || !src.bytes().all(|b| (33..=126).contains(&b))
        || src.contains(['#', '\\', '\'', '"'])
    {
        return Err(E::NaoReconhecido);
    }
    let (mut acoes, mut hashes, mut ids) = (0, 0, Vec::new());
    for par in consulta.split('&') {
        let (nome, valor) = par.split_once('=').ok_or(E::NaoReconhecido)?;
        if conferir_chave_de_consulta(nome).is_err() || conferir_valor_de_consulta(valor).is_err() {
            return Err(E::NaoReconhecido);
        }
        match nome {
            "acao" if valor == acao => acoes += 1,
            "infra_hash" if valor.len() == 64 && valor.bytes().all(|b| b.is_ascii_hexdigit()) => {
                hashes += 1;
            }
            "acao" | "acao_ajax" | "infra_hash" => return Err(E::NaoReconhecido),
            "id_procedimento" => ids.push(valor),
            _ => {}
        }
    }
    if acoes != 1 || hashes != 1 {
        return Err(E::NaoReconhecido);
    }
    match ids.as_slice() {
        [unico] if *unico == id => {}
        [_] => return Err(E::DeOutroProcesso),
        _ => return Err(E::NaoReconhecido),
    }
    Ok(format!("{}/{src}", instalacao.url_base()))
}
