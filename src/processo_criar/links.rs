//! Endereços de navegação observados nas telas de criação; sem seguir
//! redirecionamentos automaticamente e sem inferir IDs ou URLs assinadas.
use super::ErroCriar;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;

pub(super) fn id_do_link(link: &str) -> Option<&str> {
    let consulta = ConsultaDoControlador::ler(link.split('?').nth(1)?).ok()?;
    let primeiro = consulta.unico("id_tipo_procedimento").ok()?;
    (!primeiro.is_empty() && primeiro.bytes().all(|b| b.is_ascii_digit())).then_some(primeiro)
}

/// Exige exatamente uma ocorrência do parâmetro acao, com valor literal
/// esperado. Uma segunda ação diferente também torna a URL ambígua.
fn acao_unica(consulta: &str, esperada: AcaoDoSei) -> bool {
    ConsultaDoControlador::ler(consulta)
        .is_ok_and(|consulta| consulta.unico("acao") == Ok(esperada.como_na_url()))
}

pub(super) fn resolver_link(
    instalacao: &Instalacao,
    base: &str,
    referencia: &str,
    acao: AcaoDoSei,
) -> Result<String, ErroCriar> {
    if !referencia.starts_with("controlador.php?")
        || referencia.contains(['#', '\\', '\''])
        || !referencia.bytes().all(|b| (33..=126).contains(&b))
    {
        return Err(ErroCriar::Formulario);
    }
    let base = base.split('?').next().ok_or(ErroCriar::Formulario)?;
    let (pasta, _) = base.rsplit_once('/').ok_or(ErroCriar::Formulario)?;
    let url = format!("{pasta}/{referencia}");
    let partes = crate::url_observada::UrlHttps::interpretar(&url).ok_or(ErroCriar::Formulario)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::Formulario);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::Formulario);
    }
    Ok(url)
}

pub(super) fn url_do_redirecionamento(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
    acao: AcaoDoSei,
) -> Result<String, ErroCriar> {
    let absoluta = if localizacao.starts_with("https://") {
        localizacao.to_owned()
    } else {
        resolver_link(instalacao, url_envio, localizacao, acao)?
    };
    let partes =
        crate::url_observada::UrlHttps::interpretar(&absoluta).ok_or(ErroCriar::NaoConfirmado)?;
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return Err(ErroCriar::NaoConfirmado);
    }
    if !acao_unica(partes.consulta.unwrap_or_default(), acao) {
        return Err(ErroCriar::NaoConfirmado);
    }
    Ok(absoluta)
}

pub(super) fn retorno_da_criacao(
    instalacao: &Instalacao,
    localizacao: &str,
    url_envio: &str,
) -> Result<(String, String), ErroCriar> {
    // A `UnB` termina o Location com a âncora da árvore (`#ID-<id>`); o
    // fragmento não vai ao servidor e fica fora da validação.
    let localizacao = localizacao
        .split_once('#')
        .map_or(localizacao, |(url, _)| url);
    let url = url_do_redirecionamento(
        instalacao,
        localizacao,
        url_envio,
        AcaoDoSei::ProcedimentoTrabalhar,
    )
    .map_err(|_| ErroCriar::NaoConfirmado)?;
    let consulta = crate::url_observada::UrlHttps::interpretar(&url)
        .and_then(|p| p.consulta)
        .ok_or(ErroCriar::NaoConfirmado)?;
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroCriar::ConsultaDoRetornoIlegivel)?;
    let primeiro = consulta
        .unico("id_procedimento")
        .map_err(ErroCriar::IdDoRetornoIlegivel)?;
    if primeiro.is_empty() || !primeiro.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroCriar::NaoConfirmado);
    }
    let id = primeiro.to_owned();
    Ok((url, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consulta_do_controlador::ErroChaveDaConsulta;

    const ENVIO: &str = "https://sei.exemplo.test/sei/controlador.php?acao=procedimento_gerar";

    fn retorno(localizacao: &str) -> Result<(String, String), ErroCriar> {
        let instalacao = Instalacao::interpretar("https://sei.exemplo.test/sei").unwrap();
        retorno_da_criacao(&instalacao, localizacao, ENVIO)
    }

    #[test]
    fn id_do_retorno_repetido_tem_a_causa_da_chave() {
        assert!(matches!(
            retorno(
                "controlador.php?acao=procedimento_trabalhar&id_procedimento=12&id_procedimento=13"
            ),
            Err(ErroCriar::IdDoRetornoIlegivel(
                ErroChaveDaConsulta::Repetida {
                    chave: "id_procedimento"
                }
            ))
        ));
    }

    #[test]
    fn id_do_retorno_ausente_tem_a_causa_da_chave() {
        assert!(matches!(
            retorno("controlador.php?acao=procedimento_trabalhar"),
            Err(ErroCriar::IdDoRetornoIlegivel(
                ErroChaveDaConsulta::Ausente {
                    chave: "id_procedimento"
                }
            ))
        ));
    }
}
