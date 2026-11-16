//! Conferência comum dos endereços assinados que a tela do processo e a
//! árvore publicam: `controlador.php?…` relativo (a forma servida), com uma
//! `acao` igual à da operação, um `id_procedimento` igual ao aberto e um
//! `infra_hash` de 64 hex, chaves canônicas. Resolvido contra a instalação;
//! sem regex.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum EnderecoRecusado {
    NaoReconhecido,
    DeOutroProcesso,
}

fn ler_consulta_do_endereco(src: &str) -> Result<ConsultaDoControlador<'_>, EnderecoRecusado> {
    let consulta = src
        .strip_prefix("controlador.php?")
        .ok_or(EnderecoRecusado::NaoReconhecido)?;
    if src.len() > 8_192
        || !src.bytes().all(|b| (33..=126).contains(&b))
        || src.contains(['#', '\\', '\'', '"'])
    {
        return Err(EnderecoRecusado::NaoReconhecido);
    }
    ConsultaDoControlador::ler(consulta).map_err(|_| EnderecoRecusado::NaoReconhecido)
}

fn conferir_acao_e_hash(
    leitura: &ConsultaDoControlador<'_>,
    acao: AcaoDoSei,
) -> Result<(), EnderecoRecusado> {
    if leitura.acao() != Ok(acao.como_na_url()) || leitura.valores("acao_ajax").next().is_some() {
        return Err(EnderecoRecusado::NaoReconhecido);
    }
    let Ok(hash) = leitura.unico("infra_hash") else {
        return Err(EnderecoRecusado::NaoReconhecido);
    };
    if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(EnderecoRecusado::NaoReconhecido);
    }
    Ok(())
}

/// Devolve a URL absoluta na instalação.
pub(crate) fn conferir_endereco_relativo(
    instalacao: &Instalacao,
    src: &str,
    acao: AcaoDoSei,
    id: &str,
) -> Result<String, EnderecoRecusado> {
    conferir_endereco_relativo_por(instalacao, src, acao, "id_procedimento", id)
}

/// Como [`conferir_endereco_relativo`], com o id na chave `chave_do_id`
/// (`id_documento` no `Nos[i].src` de um documento, Refs #60).
pub(crate) fn conferir_endereco_relativo_por(
    instalacao: &Instalacao,
    src: &str,
    acao: AcaoDoSei,
    chave_do_id: &str,
    id: &str,
) -> Result<String, EnderecoRecusado> {
    let leitura = ler_consulta_do_endereco(src)?;
    conferir_acao_e_hash(&leitura, acao)?;
    match leitura.valores(chave_do_id).collect::<Vec<_>>().as_slice() {
        [unico] if *unico == id => {}
        [_] => return Err(EnderecoRecusado::DeOutroProcesso),
        _ => return Err(EnderecoRecusado::NaoReconhecido),
    }
    Ok(format!("{}/{src}", instalacao.url_base()))
}

/// Como [`conferir_endereco_relativo`], aceitando a segunda ocorrência
/// literalmente igual de `id_procedimento` que o SEI 4.0.9 publica nos links
/// da lista de tipos (`documento_gerar` e `documento_receber`, `UnB`).
/// Valores divergentes ou mais de duas ocorrências continuam recusados.
pub(crate) fn conferir_endereco_com_identidade_repetida(
    instalacao: &Instalacao,
    href: &str,
    acao: AcaoDoSei,
    id_procedimento: &str,
) -> Result<String, EnderecoRecusado> {
    let leitura = ler_consulta_do_endereco(href)?;
    match leitura
        .valores("id_procedimento")
        .collect::<Vec<_>>()
        .as_slice()
    {
        [unico] if *unico == id_procedimento => {}
        [primeiro, segundo] if *primeiro == id_procedimento && *segundo == id_procedimento => {}
        [unico] if *unico != id_procedimento => return Err(EnderecoRecusado::DeOutroProcesso),
        [primeiro, segundo] if *primeiro != id_procedimento || *segundo != id_procedimento => {
            return Err(EnderecoRecusado::DeOutroProcesso);
        }
        _ => return Err(EnderecoRecusado::NaoReconhecido),
    }
    conferir_acao_e_hash(&leitura, acao)?;
    Ok(format!("{}/{href}", instalacao.url_base()))
}
