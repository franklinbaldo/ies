//! Os `Location` dos dois 302 da troca, lidos como dado: o Controle
//! (`procedimento_controlar`) da unidade escolhida, no controlador da
//! instalação. Mesma regra de origem do login e das escritas
//! (`consulta_do_controlador` → `sufixo_da_mesma_origem`): relativo, ou
//! `https://`/`http://` no host do pedido anterior (o GO responde
//! `http://`). Os espaços literais do `msg` saem como `%20` (a serialização
//! do navegador); a URL devolvida é sempre `https://`.
use super::tela::conferir_consulta;
use crate::formulario_web::consulta_do_controlador;

/// URL HTTPS a pedir para `localizacao`, recebida na resposta de
/// `url_anterior`; `id_unidade` é o id escolhido na tela.
pub(crate) fn url_do_salto(
    localizacao: &str,
    url_anterior: &str,
    id_unidade: &str,
) -> Result<String, super::FalhaDepoisDoEnvio> {
    let serializada = localizacao.replace(' ', "%20");
    let consulta = consulta_do_controlador(&serializada, url_anterior)
        .ok_or(super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada)?;
    if !serializada.bytes().all(|b| (33..=126).contains(&b)) || serializada.contains(['#', '\\']) {
        return Err(super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada);
    }
    let unidade = conferir_consulta(consulta, "procedimento_controlar", hash_do_salto)
        .map_err(|_| super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada)?
        .ok_or(super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada)?;
    if unidade != id_unidade {
        return Err(super::FalhaDepoisDoEnvio::LocalizacaoDeOutraUnidade);
    }
    let base = url_anterior
        .strip_prefix("https://")
        .and_then(|r| r.split('?').next())
        .ok_or(super::FalhaDepoisDoEnvio::LocalizacaoForaDaFormaObservada)?;
    Ok(format!("https://{base}?{consulta}"))
}

/// O hash de um salto: 64 ou 192 hexadecimais (192 nas três instalações).
pub(crate) fn hash_do_salto(valor: &str) -> bool {
    matches!(valor.len(), 64 | 192) && valor.bytes().all(|b| b.is_ascii_hexdigit())
}
