//! Filtros opcionais da pesquisa: somente controles comprovados na tela servida.
//! Todos é oráculo de nomes, mas cada instalação precisa publicar os controles.

use crate::unidade_retornos::DataCivil;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FiltrosDaPesquisa<'a> {
    pub(crate) texto: Option<&'a str>,
    pub(crate) tipo: Option<&'a str>,
    pub(crate) descricao: Option<&'a str>,
    pub(crate) observacao: Option<&'a str>,
    pub(crate) assunto: Option<&'a str>,
    pub(crate) de: Option<DataCivil>,
    pub(crate) ate: Option<DataCivil>,
}

impl FiltrosDaPesquisa<'_> {
    /// Só critérios já implementados; paginação e assunto sem ID não filtram.
    pub(crate) fn tem_criterio_efetivo(&self) -> bool {
        self.tipo.is_some_and(|tipo| !tipo.trim().is_empty())
            || self.descricao.is_some()
            || self.observacao.is_some()
            || self.de.is_some()
            || self.ate.is_some()
    }
}

/// `--de`/`--ate` DD/MM/AAAA que existe no calendário (parser do clap: saída 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct DataForaDoCalendario;

pub(crate) fn data_da_cli(data: &str) -> Result<DataCivil, DataForaDoCalendario> {
    DataCivil::ler(data).ok_or(DataForaDoCalendario)
}

/// `(ano, mes, dia)` de uma data DD/MM/AAAA só com dígitos ASCII; a tupla
/// ordena como o calendário. A validade é conferida por `DataCivil::ler`.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroFiltro {
    CriterioNaoOferecidoPelaTela,
    TipoNaoOferecidoPelaTela,
    ControleDeFiltroAmbiguo,
    TextoDeFiltroInvalido,
    MaxlengthDeFiltroInvalido,
    TextoDeFiltroAcimaDoMaxlength { maxlength: u32 },
}

/// Texto de uma linha, sem alterar espaços ou caracteres significativos.
pub(crate) fn texto_da_cli(texto: &str) -> Result<String, ErroFiltro> {
    if texto.trim().is_empty() || texto.chars().any(char::is_control) {
        return Err(ErroFiltro::TextoDeFiltroInvalido);
    }
    Ok(texto.to_owned())
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

fn texto_do_controle(
    tags: &[crate::tela_html::Elemento<'_>],
    nome: &str,
    texto: &str,
) -> Result<String, ErroFiltro> {
    let campo = unico(tags, "input", nome)?;
    if campo.atributo("type") != Some("text")
        || campo.presente("disabled")
        || campo.presente("readonly")
    {
        return Err(ErroFiltro::CriterioNaoOferecidoPelaTela);
    }
    let maxlength: u32 = campo
        .atributo("maxlength")
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|v| v.parse().ok())
        .ok_or(ErroFiltro::MaxlengthDeFiltroInvalido)?;
    let texto = texto_da_cli(texto)?;
    if usize::try_from(maxlength).is_ok_and(|maximo| texto.encode_utf16().count() > maximo) {
        return Err(ErroFiltro::TextoDeFiltroAcimaDoMaxlength { maxlength });
    }
    Ok(texto)
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
    if let Some(texto) = filtros.texto {
        campos.push(("q", texto.to_owned()));
    }
    if let Some(tipo) = filtros.tipo {
        let nome = if em_documentos {
            "selSeriePesquisa"
        } else {
            "selTipoProcedimentoPesquisa"
        };
        campos.push((nome, valor_do_tipo(&tags, nome, tipo)?));
    }
    for (texto, nome) in [
        (filtros.descricao, "txtDescricaoPesquisa"),
        (filtros.observacao, "txtObservacaoPesquisa"),
    ] {
        if let Some(texto) = texto {
            campos.push((nome, texto_do_controle(&tags, nome, texto)?));
        }
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
            campos.push((nome, data.to_string()));
        }
    }
    Ok(campos)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn datas_validas_e_invalidas() {
        assert!(DataCivil::ler("29/02/2024").is_some());
        assert!(DataCivil::ler("31/12/2026").is_some());
        assert!(DataCivil::ler("29/02/2025").is_none());
        assert!(DataCivil::ler("31/11/2026").is_none());
        assert!(DataCivil::ler("01-01-2026").is_none());
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
            de: DataCivil::ler("01/01/2026"),
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
