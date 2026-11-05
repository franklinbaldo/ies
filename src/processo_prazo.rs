//! Controle de prazo da tela Web observada nas cassettes UnB/GO/UFRJ.
//! O núcleo não faz rede, não avalia JavaScript e não publica URLs assinadas.
//! Caso de testes: Todos tests/test_ciclo_prazo_010.py @323eb01.

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

impl ErroPrazo {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::OpcoesIncompativeis => "opcoes_de_prazo_incompativeis",
            Self::DataInvalida => "data_do_prazo_invalida",
            Self::DiasInvalidos => "dias_do_prazo_invalidos",
            Self::TelaNaoObservada => "tela_de_prazo_nao_observada",
            Self::OutroProcesso => "tela_de_prazo_de_outro_processo",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDoPrazo {
    SemPrazo,
    Data(String),
    Concluido,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDoPrazo {
    pub(crate) estado: EstadoDoPrazo,
    pub(crate) valor_dos_uteis: String,
}

pub(crate) fn interpretar_pedido(
    _ate: Option<&str>,
    _dias: Option<u32>,
    _excluir: bool,
) -> Result<PedidoDePrazo, ErroPrazo> {
    Err(ErroPrazo::OpcoesIncompativeis)
}

pub(crate) fn ler_tela(_html: &str, _id_procedimento: &str) -> Result<TelaDoPrazo, ErroPrazo> {
    Err(ErroPrazo::TelaNaoObservada)
}

/// Pares do usuário; os ocultos, botão e ação virão da própria tela servida.
pub(crate) fn campos_do_pedido(_pedido: &PedidoDePrazo) -> Vec<(&'static str, String)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(interpretar_pedido(None, None, false), Ok(PedidoDePrazo::Ler));
        assert_eq!(interpretar_pedido(None, None, true), Ok(PedidoDePrazo::Excluir));
        assert_eq!(
            interpretar_pedido(None, Some(15), false),
            Ok(PedidoDePrazo::Dias(15))
        );
        assert_eq!(
            interpretar_pedido(Some("15/11/2026"), Some(3), false),
            Err(ErroPrazo::OpcoesIncompativeis)
        );
        assert_eq!(interpretar_pedido(None, Some(0), false), Err(ErroPrazo::DiasInvalidos));
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
                ("txtPrazo", "".to_owned()),
                ("txtDias", "15".to_owned())
            ]
        );
        assert_eq!(
            campos_do_pedido(&PedidoDePrazo::Ate("15/11/2026".to_owned())),
            vec![
                ("rdoPrazo", "1".to_owned()),
                ("txtPrazo", "15/11/2026".to_owned()),
                ("txtDias", "".to_owned())
            ]
        );
    }
}
