use super::*;
use clap::Parser;

const RESPOSTA: &str = r#"<html><table class="pesquisaResultado">
<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal"
href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=7">00001.123456/2026-07</a>
</td></tr></table></html>"#;

fn comando(nup: &str) -> Processo {
    let cli = Cli::try_parse_from([
        "ies",
        "processo",
        nup,
        "--resultado-pesquisa-file",
        "/tmp/captura.html",
    ])
    .unwrap();
    let crate::Domain::Processo(comando) = cli.command else {
        panic!("substantivo processo esperado");
    };
    comando
}

#[test]
fn numero_opaco_exato_com_zeros_preservados() {
    assert_eq!(
        conferir(&comando("00001.123456/2026-07"), None)
            .unwrap()
            .as_str(),
        "00001.123456/2026-07"
    );
}

#[test]
fn controles_na_entrada_sao_erro_tipado_antes_do_trim() {
    assert_eq!(
        conferir(&comando("00001.123456/2026-07\n"), None),
        Err(ErroPesquisaPrivada::Numero(
            processo::NumeroProcessoInvalido::Controle
        ))
    );
}

#[test]
fn unidade_remota_nao_eh_inferida_da_captura_local() {
    assert_eq!(
        conferir(&comando("00001.123456/2026-07"), Some("PPI")),
        Err(ErroPesquisaPrivada::OpcoesIncompativeisComPesquisaPrivada)
    );
}

#[test]
fn resultado_encontrado_preserva_numero_mas_nao_abre_link() {
    let nup = processo::NumeroProcesso::novo("00001.123456/2026-07").unwrap();
    assert_eq!(selecionar(RESPOSTA, &nup), Ok(1));
}

#[test]
fn ausencia_em_uma_resposta_nao_prova_inexistencia_no_sei() {
    let nup = processo::NumeroProcesso::novo("00002.123456/2026-07").unwrap();
    assert_eq!(
        selecionar(RESPOSTA, &nup),
        Err(ErroPesquisaPrivada::NumeroNaoObservadoNaResposta)
    );
}

#[test]
fn resposta_incompleta_e_nup_duplicado_falham_com_variantes_concretas() {
    let nup = processo::NumeroProcesso::novo("00001.123456/2026-07").unwrap();
    assert_eq!(
        selecionar("<form id='frmPesquisaProtocolo'></form>", &nup),
        Err(ErroPesquisaPrivada::Resposta(
            pesquisa_web::ErroResultadoPesquisa::TabelaDePesquisaAusente
        ))
    );
    let repetida = RESPOSTA.replace(
        "</table>",
        r#"<tr class="pesquisaTituloRegistro"><td><a class="protocoloNormal"
href="controlador.php?acao=procedimento_trabalhar&amp;id_procedimento=8">00001.123456/2026-07</a>
</td></tr></table>"#,
    );
    assert_eq!(
        selecionar(&repetida, &nup),
        Err(ErroPesquisaPrivada::Resposta(
            pesquisa_web::ErroResultadoPesquisa::NumeroDePesquisaRepetido
        ))
    );
}
