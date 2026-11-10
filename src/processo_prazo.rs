//! Controle de prazo da tela Web observada nas cassettes UnB/GO/UFRJ.
//! O núcleo não faz rede, não avalia JavaScript e não publica URLs assinadas.
//! Caso de testes: `Todos` `tests/test_ciclo_prazo_010.py` @323eb01.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PedidoDePrazo {
    Ler,
    Ate(String),
    Dias(u16),
    Excluir,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroPrazo {
    OpcoesIncompativeis,
    DataInvalida,
    DiasInvalidos,
    TelaNaoObservada,
    OutroProcesso,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoPrazo {
    SemPrazo,
    Data(String),
    Dias(u16),
    Concluido,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDoPrazo {
    pub(crate) estado: EstadoDoPrazo,
    pub(crate) valor_dos_uteis: String,
}

pub(crate) fn interpretar_pedido(
    ate: Option<&str>,
    dias: Option<u32>,
    excluir: bool,
) -> Result<PedidoDePrazo, ErroPrazo> {
    let quantas = u8::from(ate.is_some()) + u8::from(dias.is_some()) + u8::from(excluir);
    if quantas > 1 {
        return Err(ErroPrazo::OpcoesIncompativeis);
    }
    if excluir {
        return Ok(PedidoDePrazo::Excluir);
    }
    if let Some(dias) = dias {
        let dias = u16::try_from(dias).map_err(|_| ErroPrazo::DiasInvalidos)?;
        if !(1..=999).contains(&dias) {
            return Err(ErroPrazo::DiasInvalidos);
        }
        return Ok(PedidoDePrazo::Dias(dias));
    }
    if let Some(data) = ate {
        if !data_valida(data) {
            return Err(ErroPrazo::DataInvalida);
        }
        return Ok(PedidoDePrazo::Ate(data.to_owned()));
    }
    Ok(PedidoDePrazo::Ler)
}

fn data_valida(valor: &str) -> bool {
    let bytes = valor.as_bytes();
    if bytes.len() != 10
        || bytes[2] != b'/'
        || bytes[5] != b'/'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| matches!(i, 2 | 5) || b.is_ascii_digit())
    {
        return false;
    }
    let Some(dia) = valor[0..2].parse::<u32>().ok() else {
        return false;
    };
    let Some(mes) = valor[3..5].parse::<u32>().ok() else {
        return false;
    };
    let Some(ano) = valor[6..10].parse::<u32>().ok() else {
        return false;
    };
    if ano == 0 {
        return false;
    }
    let bissexto = ano % 4 == 0 && (ano % 100 != 0 || ano % 400 == 0);
    let maximo = match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if bissexto => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximo).contains(&dia)
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroPrazo> {
    crate::html_atributos::valor_do_atributo(tag, nome).map_err(|_| ErroPrazo::TelaNaoObservada)
}

fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, ErroPrazo> {
    crate::html_atributos::atributo_presente(tag, nome).map_err(|_| ErroPrazo::TelaNaoObservada)
}

pub(crate) fn ler_tela(html: &str, id_procedimento: &str) -> Result<TelaDoPrazo, ErroPrazo> {
    use ErroPrazo as E;
    if html.len() > crate::MAX_HTML_BYTES {
        return Err(E::TelaNaoObservada);
    }
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| E::TelaNaoObservada)?;
    let form = crate::formulario_web::localizar_formulario(&dom, "frmControlePrazoCadastro")
        .map_err(|_| E::TelaNaoObservada)?;
    if atributo(form, "method")?.as_deref() != Some("post")
        || atributo(form, "enctype")?.is_some_and(|v| v != "application/x-www-form-urlencoded")
    {
        return Err(E::TelaNaoObservada);
    }
    let action = atributo(form, "action")?.ok_or(E::TelaNaoObservada)?;
    let action = crate::formulario_web::decodificar_atributo(&action).ok_or(E::TelaNaoObservada)?;
    let (caminho, consulta) = action.split_once('?').ok_or(E::TelaNaoObservada)?;
    if !caminho.ends_with("controlador.php") {
        return Err(E::TelaNaoObservada);
    }
    let valores = |chave: &str| {
        consulta
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(nome, _)| *nome == chave)
            .map(|(_, valor)| valor)
            .collect::<Vec<_>>()
    };
    if valores("acao") != ["controle_prazo_definir"] {
        return Err(E::TelaNaoObservada);
    }
    match valores("id_procedimento").as_slice() {
        [id] if *id == id_procedimento => {}
        [_] => return Err(E::OutroProcesso),
        _ => return Err(E::TelaNaoObservada),
    }

    ler_controles_da_tela(form, &dom, id_procedimento)
}

/// As duas opções de rádio e os campos que a tela observada sempre publica.
fn controles_completos(
    radios: &std::collections::HashSet<String>,
    vistos: &std::collections::HashSet<String>,
) -> bool {
    radios.contains("1")
        && radios.contains("2")
        && [
            "hdnInfraTipoPagina",
            "hdnIdControlePrazo",
            "txtPrazo",
            "txtDias",
            "chkSinDiasUteis",
        ]
        .iter()
        .all(|nome| vistos.contains(*nome))
}

/// Interpreta o valor observado do formulário sem inventar uma data para dias.
fn estado_dos_controles(
    marcada: Option<&str>,
    data: String,
    dias: &str,
) -> Result<EstadoDoPrazo, ErroPrazo> {
    match marcada {
        Some("1") if data_valida(&data) => Ok(EstadoDoPrazo::Data(data)),
        Some("2") => dias
            .parse::<u16>()
            .ok()
            .filter(|n| (1..=999).contains(n))
            .map(EstadoDoPrazo::Dias)
            .ok_or(ErroPrazo::TelaNaoObservada),
        Some("3") => Ok(EstadoDoPrazo::Concluido),
        None => Ok(EstadoDoPrazo::SemPrazo),
        _ => Err(ErroPrazo::TelaNaoObservada),
    }
}

/// Confere os nomes e tipos publicados na tela antes de derivar qualquer escrita.
fn ler_controles_da_tela(
    form: &tl::HTMLTag<'_>,
    dom: &tl::VDom<'_>,
    id_procedimento: &str,
) -> Result<TelaDoPrazo, ErroPrazo> {
    use ErroPrazo as E;
    let mut vistos = std::collections::HashSet::new();
    let mut radios = std::collections::HashSet::new();
    let mut marcada: Option<String> = None;
    let mut data = String::new();
    let mut dias = String::new();
    let mut valor_dos_uteis = String::new();
    let mut botoes = 0u8;
    let mut id_conferido = false;
    for tag in form
        .children()
        .all(dom.parser())
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if !matches!(elemento.as_str(), "input" | "button") {
            continue;
        }
        let Some(nome) = atributo(tag, "name")? else {
            continue;
        };
        if presente(tag, "disabled")? {
            return Err(E::TelaNaoObservada);
        }
        let tipo = atributo(tag, "type")?.unwrap_or_else(|| {
            if elemento == "button" {
                "submit"
            } else {
                "text"
            }
            .to_owned()
        });
        let tipo = tipo.to_ascii_lowercase();
        let valor = atributo(tag, "value")?;
        if nome == "rdoPrazo" {
            if elemento != "input" || tipo != "radio" {
                return Err(E::TelaNaoObservada);
            }
            let v = valor.ok_or(E::TelaNaoObservada)?;
            if !matches!(v.as_str(), "1" | "2" | "3") || !radios.insert(v.clone()) {
                return Err(E::TelaNaoObservada);
            }
            if presente(tag, "checked")? && marcada.replace(v).is_some() {
                return Err(E::TelaNaoObservada);
            }
            continue;
        }
        if !vistos.insert(nome.clone()) {
            return Err(E::TelaNaoObservada);
        }
        match (nome.as_str(), elemento.as_str(), tipo.as_str()) {
            ("hdnInfraTipoPagina" | "hdnIdControlePrazo", "input", "hidden") => {}
            ("hdnIdProcedimentos", "input", "hidden") => {
                id_conferido = true;
                if valor.as_deref() != Some(id_procedimento) {
                    return Err(E::OutroProcesso);
                }
            }
            ("txtPrazo", "input", "text") => {
                if presente(tag, "readonly")? {
                    return Err(E::TelaNaoObservada);
                }
                data = valor.unwrap_or_default();
            }
            ("txtDias", "input", "text") => {
                if presente(tag, "readonly")? {
                    return Err(E::TelaNaoObservada);
                }
                dias = valor.unwrap_or_default();
            }
            ("chkSinDiasUteis", "input", "checkbox") => {
                if presente(tag, "readonly")? {
                    return Err(E::TelaNaoObservada);
                }
                valor_dos_uteis = valor.unwrap_or_else(|| "on".to_owned());
            }
            ("sbmDefinirControlePrazo", "input" | "button", "submit") => {
                botoes += 1;
            }
            _ => return Err(E::TelaNaoObservada),
        }
    }
    if !id_conferido || botoes != 1 || !controles_completos(&radios, &vistos) {
        return Err(E::TelaNaoObservada);
    }
    let estado = estado_dos_controles(marcada.as_deref(), data, &dias)?;
    Ok(TelaDoPrazo {
        estado,
        valor_dos_uteis,
    })
}

/// Campos do usuário para a derivação segura do formulário observado.
/// Dias civis desde 1970-01-01 de uma data `DD/MM/AAAA` já validada
/// (algoritmo de Howard Hinnant, sem dependência de calendário).
fn dias_civis(data: &str) -> Option<i64> {
    if !data_valida(data) {
        return None;
    }
    let dia: i64 = data[0..2].parse().ok()?;
    let mes: i64 = data[3..5].parse().ok()?;
    let ano: i64 = data[6..10].parse().ok()?;
    let a = if mes <= 2 { ano - 1 } else { ano };
    let era = a.div_euclid(400);
    let ano_da_era = a - era * 400;
    let mes_deslocado = (mes + 9) % 12;
    let dia_do_ano = (153 * mes_deslocado + 2) / 5 + dia - 1;
    let dia_da_era = ano_da_era * 365 + ano_da_era / 4 - ano_da_era / 100 + dia_do_ano;
    Some(era * 146_097 + dia_da_era - 719_468)
}

/// A releitura confirma o pedido? Para `--dias N`, o SEI da `UnB` guarda e
/// mostra a data certa calculada (cassette processo-prazo-unb, interações
/// 19→21; live de 10/11/2026: `19/11/2026`). Aceita essa data quando fica a
/// N dias corridos de hoje, com folga de um dia para o fuso de Brasília.
/// `hoje` é o dia civil UTC em dias desde 1970-01-01.
pub(crate) fn releitura_confirma(
    pedido: &PedidoDePrazo,
    estado: &EstadoDoPrazo,
    hoje: i64,
) -> bool {
    match (pedido, estado) {
        (PedidoDePrazo::Ate(data_pedida), EstadoDoPrazo::Data(data_relida)) => {
            data_pedida == data_relida
        }
        (PedidoDePrazo::Dias(n), EstadoDoPrazo::Dias(lidos)) => n == lidos,
        (PedidoDePrazo::Dias(n), EstadoDoPrazo::Data(lida)) => {
            dias_civis(lida).is_some_and(|d| (d - hoje - i64::from(*n)).abs() <= 1)
        }
        _ => false,
    }
}

pub(crate) fn campos_do_pedido(pedido: &PedidoDePrazo) -> Vec<(&'static str, String)> {
    let (opcao, data, dias) = match pedido {
        PedidoDePrazo::Ate(data) => ("1", data.clone(), String::new()),
        PedidoDePrazo::Dias(dias) => ("2", String::new(), dias.to_string()),
        PedidoDePrazo::Ler | PedidoDePrazo::Excluir => return Vec::new(),
    };
    vec![
        ("rdoPrazo", opcao.to_owned()),
        ("txtPrazo", data),
        ("txtDias", dias),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dias_viram_data_certa_na_releitura_da_unb() {
        // 10/11/2026 UTC; o SEI (09/11 em Brasília) gravou 19/11/2026.
        let hoje = dias_civis("10/11/2026").unwrap();
        let dez = PedidoDePrazo::Dias(10);
        for aceita in ["19/11/2026", "20/11/2026", "21/11/2026"] {
            assert!(
                releitura_confirma(&dez, &EstadoDoPrazo::Data(aceita.into()), hoje),
                "{aceita}"
            );
        }
        for recusada in ["18/11/2026", "22/11/2026", "29/10/2026"] {
            assert!(
                !releitura_confirma(&dez, &EstadoDoPrazo::Data(recusada.into()), hoje),
                "{recusada}"
            );
        }
        assert!(releitura_confirma(&dez, &EstadoDoPrazo::Dias(10), hoje));
        assert!(!releitura_confirma(&dez, &EstadoDoPrazo::SemPrazo, hoje));
        let ate = PedidoDePrazo::Ate("19/11/2026".into());
        assert!(releitura_confirma(
            &ate,
            &EstadoDoPrazo::Data("19/11/2026".into()),
            hoje
        ));
        assert!(!releitura_confirma(
            &ate,
            &EstadoDoPrazo::Data("20/11/2026".into()),
            hoje
        ));
        assert_eq!(dias_civis("01/01/1970"), Some(0));
        assert_eq!(
            dias_civis("01/03/2024").unwrap() - dias_civis("28/02/2024").unwrap(),
            2
        );
    }

    // Fixture sintética da forma observada na UnB, sem URL de sessão real.
    fn tela(opcao: &str, data: &str) -> String {
        let radio = |valor| if opcao == valor { " checked" } else { "" };
        format!(
            "<html><body><form id='frmControlePrazoCadastro' method='post' \
             action='controlador.php?acao=controle_prazo_definir&id_procedimento=10&infra_hash=SINTETICO'>\
             <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
             <button type='submit' name='sbmDefinirControlePrazo' value='Salvar'>Salvar</button>\
             <input type='radio' name='rdoPrazo' value='1'{}>\
             <input type='text' name='txtPrazo' value='{data}'>\
             <input type='radio' name='rdoPrazo' value='2'{}>\
             <input type='text' name='txtDias' value=''>\
             <input type='checkbox' name='chkSinDiasUteis'>\
             <input type='radio' name='rdoPrazo' value='3'{}>\
             <input type='hidden' name='hdnIdControlePrazo' value=''>\
             <input type='hidden' name='hdnIdProcedimentos' value='10'>\
             </form></body></html>",
            radio("1"),
            radio("2"),
            radio("3"),
        )
    }

    // Casos de Todos tests/test_ciclo_prazo_010.py @323eb01.
    #[test]
    fn pedido_data_bissexta_valida_e_data_inexistente_recusada() {
        assert_eq!(
            interpretar_pedido(Some("29/02/2028"), None, false),
            Ok(PedidoDePrazo::Ate("29/02/2028".to_owned()))
        );
        assert_eq!(
            interpretar_pedido(Some("29/02/2027"), None, false),
            Err(ErroPrazo::DataInvalida)
        );
    }

    #[test]
    fn opcoes_exclusivas_e_dias_positivos() {
        assert_eq!(
            interpretar_pedido(None, None, false),
            Ok(PedidoDePrazo::Ler)
        );
        assert_eq!(
            interpretar_pedido(None, None, true),
            Ok(PedidoDePrazo::Excluir)
        );
        assert_eq!(
            interpretar_pedido(None, Some(15), false),
            Ok(PedidoDePrazo::Dias(15))
        );
        assert_eq!(
            interpretar_pedido(Some("15/11/2026"), Some(3), false),
            Err(ErroPrazo::OpcoesIncompativeis)
        );
        assert_eq!(
            interpretar_pedido(None, Some(0), false),
            Err(ErroPrazo::DiasInvalidos)
        );
    }

    #[test]
    fn tela_le_prazo_atual_sem_escrita_e_concluido() {
        assert_eq!(
            ler_tela(&tela("1", "30/10/2026"), "10").unwrap().estado,
            EstadoDoPrazo::Data("30/10/2026".to_owned())
        );
        assert_eq!(
            ler_tela(&tela("3", ""), "10").unwrap().estado,
            EstadoDoPrazo::Concluido
        );
        assert_eq!(
            ler_tela(&tela("", ""), "10").unwrap().estado,
            EstadoDoPrazo::SemPrazo
        );
    }

    #[test]
    fn tela_le_prazo_em_dias_sem_inferir_data_de_calendario() {
        let por_dias =
            tela("2", "").replace("name='txtDias' value=''", "name='txtDias' value='15'");
        assert_eq!(
            ler_tela(&por_dias, "10").unwrap().estado,
            EstadoDoPrazo::Dias(15)
        );
    }

    #[test]
    fn processo_estranho_e_controles_duplicados_falham_fechado() {
        assert_eq!(
            ler_tela(&tela("1", "30/10/2026"), "11"),
            Err(ErroPrazo::OutroProcesso)
        );
        let duplicada = tela("", "").replace(
            "<input type='text' name='txtDias'",
            "<input type='hidden' name='txtDias' value='3'><input type='text' name='txtDias'",
        );
        assert_eq!(ler_tela(&duplicada, "10"), Err(ErroPrazo::TelaNaoObservada));
    }

    #[test]
    fn dias_limpam_data_anterior_e_nao_enviam_uteis() {
        assert_eq!(
            campos_do_pedido(&PedidoDePrazo::Dias(15)),
            vec![
                ("rdoPrazo", "2".to_owned()),
                ("txtPrazo", String::new()),
                ("txtDias", "15".to_owned())
            ]
        );
        assert_eq!(
            campos_do_pedido(&PedidoDePrazo::Ate("15/11/2026".to_owned())),
            vec![
                ("rdoPrazo", "1".to_owned()),
                ("txtPrazo", "15/11/2026".to_owned()),
                ("txtDias", String::new())
            ]
        );
    }
}
