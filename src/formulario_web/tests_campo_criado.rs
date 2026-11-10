//! RED/GREEN do envio por script que cria um oculto (`selecionarUnidade()`
//! da tela Trocar Unidade, #205): o campo criado sai no fim do corpo, com
//! o valor da operação; publicado pela tela, é recusa. HTML sintético.
use super::tests::{instalacao, tela_iso};
use super::*;

const OPERACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "infra_trocar_unidade",
    formulario: "frmInfraSelecaoUnidade",
    envio: EnvioDoFormulario::SubmitPorScriptComCampoCriado("selInfraUnidades"),
    campos_ocultos_exigidos: &["hdnInfraItensSelecionados"],
    campos_ocultos_reescritos: &["hdnInfraItensSelecionados"],
    campos_usuario: &["chkInfraItem", "txtInfraSiglaUnidade"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::UnidadeAtual,
    },
};

const HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn url() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=infra_trocar_unidade&infra_hash={HASH}"
    )
}

fn html(interior: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>TESTE</a>\
         <form id='frmInfraSelecaoUnidade' method='post' \
         action='controlador.php?acao=infra_trocar_unidade&acao_origem=infra_trocar_unidade&infra_hash={HASH}'>\
         {interior}</form></html>"
    )
}

const INTERIOR: &str = "<input type='hidden' name='hdnInfraTipoPagina' value='1'>\
    <button type='submit' name='sbmPesquisar' value='Pesquisar'>P</button>\
    <input type='text' name='txtInfraSiglaUnidade' value=''>\
    <input type='radio' name='chkInfraItem' value='1' checked='checked'>\
    <input type='radio' name='chkInfraItem' value='2'>\
    <input type='hidden' name='hdnInfraItensSelecionados' value='1'>";

fn derivar(
    interior: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = url();
    let pagina = html(interior);
    derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, &pagina),
        "TESTE",
        campos,
    )
}

const CAMPOS: [(&str, &str); 4] = [
    ("chkInfraItem", "2"),
    ("txtInfraSiglaUnidade", ""),
    ("hdnInfraItensSelecionados", "2"),
    ("selInfraUnidades", "2"),
];

#[test]
fn campo_criado_pelo_script_sai_no_fim_sem_botao() {
    let s = derivar(INTERIOR, &CAMPOS).unwrap();
    assert_eq!(
        s.nomes_dos_campos(),
        [
            "hdnInfraTipoPagina",
            "txtInfraSiglaUnidade",
            "chkInfraItem",
            "hdnInfraItensSelecionados",
            "selInfraUnidades"
        ]
    );
    assert_eq!(
        String::from_utf8(s.corpo().to_vec()).unwrap(),
        "hdnInfraTipoPagina=1&txtInfraSiglaUnidade=&chkInfraItem=2&hdnInfraItensSelecionados=2&selInfraUnidades=2"
    );
}

#[test]
fn campo_criado_sem_valor_da_operacao_nao_e_inventado() {
    let s = derivar(INTERIOR, &CAMPOS[..3]).unwrap();
    assert!(!s.nomes_dos_campos().iter().any(|n| n == "selInfraUnidades"));
}

#[test]
fn campo_criado_ja_publicado_pela_tela_e_recusado() {
    let interior = format!("{INTERIOR}<input type='hidden' name='selInfraUnidades' value='9'>");
    assert_eq!(
        derivar(&interior, &CAMPOS).map(|_| ()),
        Err(ErroSubmissaoFormulario::CampoCriadoPeloScriptJaPublicado {
            campo: "selInfraUnidades".into()
        })
    );
}

#[test]
fn campo_criado_repetido_na_operacao_e_recusado() {
    let mut campos = CAMPOS.to_vec();
    campos.push(("selInfraUnidades", "1"));
    assert_eq!(
        derivar(INTERIOR, &campos).map(|_| ()),
        Err(ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: "selInfraUnidades".into()
        })
    );
}
