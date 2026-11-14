//! Filtros opcionais da pesquisa: somente controles comprovados na tela servida.
//! Todos é oráculo de nomes, mas cada instalação precisa publicar os controles.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FiltrosDaPesquisa<'a> {
    pub(crate) tipo: Option<&'a str>,
    pub(crate) assunto: Option<&'a str>,
    pub(crate) de: Option<&'a str>,
    pub(crate) ate: Option<&'a str>,
}

/// `--de`/`--ate` DD/MM/AAAA que existe no calendário (parser do clap: saída 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct DataForaDoCalendario;

pub(crate) fn data_da_cli(data: &str) -> Result<String, DataForaDoCalendario> {
    if data_valida(data) {
        Ok(data.to_owned())
    } else {
        Err(DataForaDoCalendario)
    }
}

/// `(ano, mes, dia)` de uma data DD/MM/AAAA só com dígitos ASCII; a tupla
/// ordena como o calendário. Não confere o calendário: ver `data_valida`.
pub(crate) fn ano_mes_dia(data: &str) -> Option<(u32, u32, u32)> {
    let (dia, resto) = data.split_once('/')?;
    let (mes, ano) = resto.split_once('/')?;
    let numero = |texto: &str, digitos: usize| {
        (texto.len() == digitos && texto.bytes().all(|b| b.is_ascii_digit()))
            .then(|| texto.parse::<u32>().ok())
            .flatten()
    };
    Some((numero(ano, 4)?, numero(mes, 2)?, numero(dia, 2)?))
}

/// Datas DD/MM/AAAA, sem normalização silenciosa e sem dependências novas.
pub(crate) fn data_valida(data: &str) -> bool {
    crate::unidade_retornos::data_valida(data)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroFiltro {
    CriterioNaoOferecidoPelaTela,
    TipoNaoOferecidoPelaTela,
    ControleDeFiltroAmbiguo,
}

fn unico<'a>(
    tags: &[crate::tela_html::Elemento<'a>],
    elemento: &str,
    nome: &str,
) -> Result<crate::tela_html::Elemento<'a>, ErroFiltro> {
    let mut encontrado = None;
    for tag in tags {
        if tag.nome().eq_ignore_ascii_case(elemento) && tag.atributo("name") == Some(nome) {
            if encontrado.is_some() {
                return Err(ErroFiltro::ControleDeFiltroAmbiguo);
            }
            encontrado = Some(*tag);
        }
    }
    encontrado.ok_or(ErroFiltro::CriterioNaoOferecidoPelaTela)
}

/// Extrai o value correspondente ao nome *exatamente* mostrado numa option.
/// Não usa índice, substring, normalização nem inventa IDs.
fn valor_do_tipo(
    tags: &[crate::tela_html::Elemento<'_>],
    nome_campo: &str,
    tipo: &str,
) -> Result<String, ErroFiltro> {
    let select = unico(tags, "select", nome_campo)?;
    let mut escolhidos = Vec::new();
    for tag in select.descendentes() {
        if !tag.nome().eq_ignore_ascii_case("option") || tag.atributo("disabled").is_some() {
            continue;
        }
        if tag.texto() == tipo {
            let valor = tag
                .atributo("value")
                .filter(|v| !v.is_empty() && *v != "null")
                .ok_or(ErroFiltro::TipoNaoOferecidoPelaTela)?;
            escolhidos.push(valor);
        }
    }
    match escolhidos.as_slice() {
        [] => Err(ErroFiltro::TipoNaoOferecidoPelaTela),
        [valor] => Ok((*valor).to_owned()),
        _ => Err(ErroFiltro::ControleDeFiltroAmbiguo),
    }
}

/// Pares próprios da operação, conservados até o POST do formulário observado.
/// O assunto exige ID de autocompletar: texto isolado não filtra no SEI.
pub(crate) fn derivar_campos(
    form: crate::tela_html::Elemento<'_>,
    em_documentos: bool,
    filtros: FiltrosDaPesquisa<'_>,
) -> Result<Vec<(&'static str, String)>, ErroFiltro> {
    let tags: Vec<_> = form.descendentes().collect();
    let mut campos = Vec::new();
    if let Some(tipo) = filtros.tipo {
        let nome = if em_documentos {
            "selSeriePesquisa"
        } else {
            "selTipoProcedimentoPesquisa"
        };
        campos.push((nome, valor_do_tipo(&tags, nome, tipo)?));
    }
    if filtros.assunto.is_some() {
        // txtAssunto + hdnIdAssunto são autocompletar; requerem resolução
        // observada do ID. Não enviar texto que o servidor ignoraria.
        return Err(ErroFiltro::CriterioNaoOferecidoPelaTela);
    }
    for (data, nome) in [(filtros.de, "txtDataInicio"), (filtros.ate, "txtDataFim")] {
        if let Some(data) = data {
            let campo = unico(&tags, "input", nome)?;
            if campo.atributo("type").is_some_and(|v| v != "text") {
                return Err(ErroFiltro::CriterioNaoOferecidoPelaTela);
            }
            campos.push((nome, data.to_owned()));
        }
    }
    Ok(campos)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn datas_validas_e_invalidas() {
        assert!(data_valida("29/02/2024"));
        assert!(data_valida("31/12/2026"));
        assert!(!data_valida("29/02/2025"));
        assert!(!data_valida("31/11/2026"));
        assert!(!data_valida("01-01-2026"));
    }
    #[test]
    fn tipo_exato_e_recusa_de_ausencia() {
        let html = crate::tela_html::HtmlServido::de_teste(
            "<form><select name='selTipoProcedimentoPesquisa'><option value='null'>-</option><option value='7'>Administrativo</option><option value='8'>Judicial</option></select><input name='txtDataInicio' type='text'></form>",
        );
        let servido = html;
        let dom = crate::tela_html::arvore(&servido);
        let form = dom.elementos("form").next().unwrap();
        let filtros = FiltrosDaPesquisa {
            tipo: Some("Judicial"),
            de: Some("01/01/2026"),
            ..Default::default()
        };
        assert_eq!(
            derivar_campos(form, false, filtros).unwrap(),
            vec![
                ("selTipoProcedimentoPesquisa", "8".to_owned()),
                ("txtDataInicio", "01/01/2026".to_owned())
            ]
        );
        assert_eq!(
            derivar_campos(form, true, filtros),
            Err(ErroFiltro::CriterioNaoOferecidoPelaTela)
        );
        assert_eq!(
            derivar_campos(
                form,
                false,
                FiltrosDaPesquisa {
                    assunto: Some("Assunto"),
                    ..Default::default()
                }
            ),
            Err(ErroFiltro::CriterioNaoOferecidoPelaTela)
        );
    }
}
