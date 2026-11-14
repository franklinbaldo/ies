//! Controle de prazo da tela Web observada nas cassettes UnB/GO/UFRJ.
//! O núcleo não faz rede, não avalia JavaScript e não publica URLs assinadas.
//! Caso de testes: `Todos` `tests/test_ciclo_prazo_010.py` @323eb01.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroConsultaDoControlador, ErroParametroDaUrl,
};

use crate::unidade_retornos::DataCivil;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PedidoDePrazo {
    Ler,
    Ate(DataCivil),
    Dias(u16),
    Excluir,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroPrazo {
    OpcoesIncompativeis,
    DataInvalida,
    DiasInvalidos,
    /// O `action` do formulário não é `controle_prazo_definir` em `controlador.php`.
    FormularioDeOutroDestino,
    /// A consulta do `action` não se lê em pares `chave=valor`.
    ConsultaDoDestinoIlegivel(#[source] ErroConsultaDoControlador),
    /// O `action` não tem exatamente um `acao` aceito nem um `id_procedimento`.
    ChaveDoDestinoIlegivel(#[source] ErroParametroDaUrl),
    AcaoDoDestinoIlegivel(#[source] crate::consulta_do_controlador::ErroAcaoDaUrl),
    /// O formulário não publica as opções `rdoPrazo`: não há estado a ler.
    OpcoesDoPrazoAusentes,
    /// A opção marcada ou o valor de dias não formam um prazo.
    PrazoAtualIlegivel,
    /// O formulário não traz `hdnIdProcedimentos`: o processo não é conferido.
    IdDoProcessoAusenteNoFormulario,
    /// O formulário de cadastro do prazo não foi localizado de forma única.
    FormularioDoPrazoAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    OutroProcesso,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoPrazo {
    SemPrazo,
    Data(DataCivil),
    Dias(u16),
    Concluido,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDoPrazo {
    pub(crate) estado: EstadoDoPrazo,
    pub(crate) valor_dos_uteis: String,
}

/// `--ate`: o clap recusa a data inexistente (saída 2) antes da sessão.
pub(crate) fn data_do_prazo(valor: &str) -> Result<DataCivil, ErroPrazo> {
    DataCivil::ler(valor).ok_or(ErroPrazo::DataInvalida)
}

/// `--dias`: de 1 a 999; o clap recusa o resto (saída 2).
pub(crate) fn dias_do_prazo(valor: &str) -> Result<u16, ErroPrazo> {
    match valor.parse::<u16>() {
        Ok(dias @ 1..=999) => Ok(dias),
        _ => Err(ErroPrazo::DiasInvalidos),
    }
}

pub(crate) fn interpretar_pedido(
    ate: Option<DataCivil>,
    dias: Option<u16>,
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
        if !(1..=999).contains(&dias) {
            return Err(ErroPrazo::DiasInvalidos);
        }
        return Ok(PedidoDePrazo::Dias(dias));
    }
    if let Some(data) = ate {
        return Ok(PedidoDePrazo::Ate(data));
    }
    Ok(PedidoDePrazo::Ler)
}

/// Lê a tela do prazo. Só o que protege a escrita é conferido: o `action`
/// aponta para `controle_prazo_definir` deste processo e
/// `hdnIdProcedimentos` é o dele. A forma dos controles fica com a
/// derivação do formulário.
pub(crate) fn ler_tela(
    html: &crate::tela_html::HtmlServido,
    id_procedimento: &str,
) -> Result<TelaDoPrazo, ErroPrazo> {
    let dom = crate::tela_html::arvore(html);
    let form = crate::formulario_web::localizar_formulario(&dom, "frmControlePrazoCadastro")
        .map_err(ErroPrazo::FormularioDoPrazoAusente)?;
    conferir_destino(form, id_procedimento)?;
    ler_controles_da_tela(form, id_procedimento)
}

/// O `action` do formulário é `controlador.php?acao=controle_prazo_definir`
/// com o `id_procedimento` pedido; outro destino recusa antes de qualquer escrita.
fn conferir_destino(
    form: crate::tela_html::Elemento<'_>,
    id_procedimento: &str,
) -> Result<(), ErroPrazo> {
    let action = form
        .atributo("action")
        .ok_or(ErroPrazo::FormularioDeOutroDestino)?;
    let (caminho, consulta) = action
        .split_once('?')
        .ok_or(ErroPrazo::FormularioDeOutroDestino)?;
    if !caminho.ends_with("controlador.php") {
        return Err(ErroPrazo::FormularioDeOutroDestino);
    }
    let consulta =
        ConsultaDoControlador::ler(consulta).map_err(ErroPrazo::ConsultaDoDestinoIlegivel)?;
    let acao = consulta.acao().map_err(ErroPrazo::AcaoDoDestinoIlegivel)?;
    if acao != AcaoDoSei::ControlePrazoDefinir.como_na_url() {
        return Err(ErroPrazo::FormularioDeOutroDestino);
    }
    let id = consulta
        .unico("id_procedimento")
        .map_err(ErroPrazo::ChaveDoDestinoIlegivel)?;
    if id != id_procedimento {
        return Err(ErroPrazo::OutroProcesso);
    }
    Ok(())
}

/// O estado lido das opções publicadas; sem as opções, não há o que ler.
fn estado_dos_controles(
    opcoes_publicadas: bool,
    marcada: Option<&str>,
    data: &str,
    dias: &str,
) -> Result<EstadoDoPrazo, ErroPrazo> {
    if !opcoes_publicadas {
        return Err(ErroPrazo::OpcoesDoPrazoAusentes);
    }
    match marcada {
        Some("1") => DataCivil::ler(data)
            .map(EstadoDoPrazo::Data)
            .ok_or(ErroPrazo::PrazoAtualIlegivel),
        Some("2") => dias
            .parse::<u16>()
            .ok()
            .filter(|n| (1..=999).contains(n))
            .map(EstadoDoPrazo::Dias)
            .ok_or(ErroPrazo::PrazoAtualIlegivel),
        Some("3") => Ok(EstadoDoPrazo::Concluido),
        None => Ok(EstadoDoPrazo::SemPrazo),
        Some(_) => Err(ErroPrazo::PrazoAtualIlegivel),
    }
}

/// Os valores atuais publicados no formulário e o id do processo.
fn ler_controles_da_tela(
    form: crate::tela_html::Elemento<'_>,
    id_procedimento: &str,
) -> Result<TelaDoPrazo, ErroPrazo> {
    let mut opcoes_publicadas = false;
    let mut marcada: Option<&str> = None;
    let mut data = "";
    let mut dias = "";
    let mut valor_dos_uteis = "on".to_owned();
    let mut id_conferido = false;
    for tag in form.descendentes() {
        let Some(nome) = tag.atributo("name") else {
            continue;
        };
        let valor = tag.atributo("value");
        match nome {
            "rdoPrazo" => {
                opcoes_publicadas = true;
                if tag.presente("checked") {
                    marcada = valor;
                }
            }
            "hdnIdProcedimentos" => {
                if valor != Some(id_procedimento) {
                    return Err(ErroPrazo::OutroProcesso);
                }
                id_conferido = true;
            }
            "txtPrazo" => data = valor.unwrap_or_default(),
            "txtDias" => dias = valor.unwrap_or_default(),
            "chkSinDiasUteis" => {
                if let Some(v) = valor {
                    v.clone_into(&mut valor_dos_uteis);
                }
            }
            _ => {}
        }
    }
    if !id_conferido {
        return Err(ErroPrazo::IdDoProcessoAusenteNoFormulario);
    }
    let estado = estado_dos_controles(opcoes_publicadas, marcada, data, dias)?;
    Ok(TelaDoPrazo {
        estado,
        valor_dos_uteis,
    })
}

/// Campos do usuário para a derivação segura do formulário observado.
/// Dias civis desde 1970-01-01 de uma data civil.
fn dias_civis(valor: DataCivil) -> Option<i64> {
    let mes = time::Month::try_from(valor.mes_ano().mes()).ok()?;
    let data =
        time::Date::from_calendar_date(i32::from(valor.mes_ano().ano()), mes, valor.dia()).ok()?;
    Some((data - time::OffsetDateTime::UNIX_EPOCH.date()).whole_days())
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
            dias_civis(*lida).is_some_and(|d| (d - hoje - i64::from(*n)).abs() <= 1)
        }
        _ => false,
    }
}

pub(crate) fn campos_do_pedido(pedido: &PedidoDePrazo) -> Vec<(&'static str, String)> {
    let (opcao, data, dias) = match pedido {
        PedidoDePrazo::Ate(data) => ("1", data.to_string(), String::new()),
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
    fn destino_com_par_sem_igual_ou_id_repetido_recusa_com_a_causa() {
        let html = |action: &str| {
            format!("<form id='frmControlePrazoCadastro' action='{action}' method='post'></form>")
        };
        assert_eq!(
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&html(
                    "controlador.php?acao=controle_prazo_definir&amp;lixo&amp;id_procedimento=12"
                )),
                "12"
            ),
            Err(ErroPrazo::ConsultaDoDestinoIlegivel(
                ErroConsultaDoControlador::ParSemIgual
            ))
        );
        assert_eq!(
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&html(
                    "controlador.php?acao=controle_prazo_definir&amp;id_procedimento=12&amp;id_procedimento=12"
                )),
                "12"
            ),
            Err(ErroPrazo::ChaveDoDestinoIlegivel(
                ErroParametroDaUrl::ParametroRepetido("id_procedimento")
            ))
        );
    }

    #[test]
    fn dias_viram_data_certa_na_releitura_da_unb() {
        let d = |s| DataCivil::ler(s).unwrap();
        // 10/11/2026 UTC; o SEI (09/11 em Brasília) gravou 19/11/2026.
        let hoje = dias_civis(d("10/11/2026")).unwrap();
        let dez = PedidoDePrazo::Dias(10);
        for aceita in ["19/11/2026", "20/11/2026", "21/11/2026"] {
            assert!(
                releitura_confirma(&dez, &EstadoDoPrazo::Data(d(aceita)), hoje),
                "{aceita}"
            );
        }
        for recusada in ["18/11/2026", "22/11/2026", "29/10/2026"] {
            assert!(
                !releitura_confirma(&dez, &EstadoDoPrazo::Data(d(recusada)), hoje),
                "{recusada}"
            );
        }
        assert!(releitura_confirma(&dez, &EstadoDoPrazo::Dias(10), hoje));
        assert!(!releitura_confirma(&dez, &EstadoDoPrazo::SemPrazo, hoje));
        let ate = PedidoDePrazo::Ate(d("19/11/2026"));
        assert!(releitura_confirma(
            &ate,
            &EstadoDoPrazo::Data(d("19/11/2026")),
            hoje
        ));
        assert!(!releitura_confirma(
            &ate,
            &EstadoDoPrazo::Data(d("20/11/2026")),
            hoje
        ));
        assert_eq!(dias_civis(d("01/01/1970")), Some(0));
        assert_eq!(
            dias_civis(d("01/03/2024")).unwrap() - dias_civis(d("28/02/2024")).unwrap(),
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
        let bissexta = DataCivil::ler("29/02/2028").unwrap();
        assert_eq!(data_do_prazo("29/02/2028"), Ok(bissexta));
        assert_eq!(
            interpretar_pedido(Some(bissexta), None, false),
            Ok(PedidoDePrazo::Ate(bissexta))
        );
        assert_eq!(data_do_prazo("29/02/2027"), Err(ErroPrazo::DataInvalida));
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
            interpretar_pedido(Some(DataCivil::ler("15/11/2026").unwrap()), Some(3), false),
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
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&tela("1", "30/10/2026")),
                "10"
            )
            .unwrap()
            .estado,
            EstadoDoPrazo::Data(DataCivil::ler("30/10/2026").unwrap())
        );
        assert_eq!(
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&tela("3", "")),
                "10"
            )
            .unwrap()
            .estado,
            EstadoDoPrazo::Concluido
        );
        assert_eq!(
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&tela("", "")),
                "10"
            )
            .unwrap()
            .estado,
            EstadoDoPrazo::SemPrazo
        );
    }

    #[test]
    fn tela_le_prazo_em_dias_sem_inferir_data_de_calendario() {
        let por_dias =
            tela("2", "").replace("name='txtDias' value=''", "name='txtDias' value='15'");
        assert_eq!(
            ler_tela(&crate::tela_html::HtmlServido::de_teste(&por_dias), "10")
                .unwrap()
                .estado,
            EstadoDoPrazo::Dias(15)
        );
    }

    #[test]
    fn processo_estranho_falha_fechado() {
        assert_eq!(
            ler_tela(
                &crate::tela_html::HtmlServido::de_teste(&tela("1", "30/10/2026")),
                "11"
            ),
            Err(ErroPrazo::OutroProcesso)
        );
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
            campos_do_pedido(&PedidoDePrazo::Ate(DataCivil::ler("15/11/2026").unwrap())),
            vec![
                ("rdoPrazo", "1".to_owned()),
                ("txtPrazo", "15/11/2026".to_owned()),
                ("txtDias", String::new())
            ]
        );
    }
}
