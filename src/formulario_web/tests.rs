//! RED/GREEN da derivação: HTML sintético modelado no formulário servido.
use super::*;
use crate::erro_html::ErroHtml;

pub(crate) const HASH_TELA: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_FORM: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(crate) const OPERACAO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "protocolo_pesquisar",
    formulario: "frmPesquisaProtocolo",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmPesquisar"),
    campos_ocultos_exigidos: &["hdnInfraTipoPagina", "hdnInicio"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["rdoPesquisarEm", "txtProtocoloPesquisa"],
    resultado: ResultadoDoEnvio::Marcador(MarcadorDeResultado {
        elemento: "table",
        classe: "pesquisaResultado",
    }),
    unidade: ConferenciaDaUnidade::NaTelaDoFormulario,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

pub(crate) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

pub(crate) fn url_servida() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisar&infra_hash={HASH_TELA}"
    )
}

pub(crate) fn tela_iso<'a>(url: &'a str, html: &'a str) -> TelaServida<'a> {
    TelaServida {
        url,
        html,
        codificacao: CodificacaoDaTela::Iso88591,
    }
}

fn action_padrao() -> String {
    format!(
        "controlador.php?acao=protocolo_pesquisar&amp;acao_origem=protocolo_pesquisar&amp;infra_hash={HASH_FORM}#ancoraBarraPesquisa"
    )
}

const INTERIOR: &str = "<input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='1'>\
    <button type='submit' id='sbmPesquisar' name='sbmPesquisar' value='Pesquisar'>Pesquisar</button>\
    <button type='button' id='sbmReset' name='sbmReset' value='Limpar'>Limpar</button>\
    <input type='radio' id='optProcessos' name='rdoPesquisarEm' value='P'>\
    <input type='radio' id='optDocumentos' name='rdoPesquisarEm' value='D'>\
    <input type='text' id='txtProtocoloPesquisa' name='txtProtocoloPesquisa' value=''>\
    <select id='selData' name='selData'><option value='0'>x</option></select>\
    <input type='hidden' id='hdnInicio' name='hdnInicio' value='0'>";

fn pagina(atributos: &str, interior: &str) -> String {
    format!(
        "<html><a id='lnkInfraUnidade'>UNIDADE  TESTE</a>\
         <form id='frmPesquisaProtocolo' name='frmPesquisaProtocolo' method='post' {atributos}>\
         {interior}</form>\
         <input type='hidden' id='hdnInfraPrefixoCookie' name='hdnInfraPrefixoCookie' value='x'></html>"
    )
}

fn padrao() -> String {
    pagina(&format!("action='{}'", action_padrao()), INTERIOR)
}

fn com_action(action: &str) -> String {
    pagina(&format!("action='{action}'"), INTERIOR)
}

fn derivar_html(
    html: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    let url = url_servida();
    derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, html),
        "UNIDADE TESTE",
        campos,
    )
}

fn erro(html: &str, campos: &[(&str, &str)]) -> ErroSubmissaoFormulario {
    derivar_html(html, campos).unwrap_err()
}

fn campo(nome: &str) -> String {
    nome.to_owned()
}

#[test]
fn deriva_ocultos_exatos_campos_e_botao_na_ordem_do_documento() {
    let s = derivar_html(
        &padrao(),
        &[("txtProtocoloPesquisa", "0001"), ("rdoPesquisarEm", "P")],
    )
    .unwrap();
    assert_eq!(SubmissaoDerivada::METODO, "POST");
    assert_eq!(
        s.url(),
        format!(
            "https://sei.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisar&acao_origem=protocolo_pesquisar&infra_hash={HASH_FORM}"
        )
    );
    assert_eq!(s.origem(), "https://sei.exemplo.gov.br");
    assert_eq!(s.referer(), url_servida());
    assert_eq!(
        SubmissaoDerivada::TIPO_DE_CONTEUDO,
        "application/x-www-form-urlencoded"
    );
    assert_eq!(
        s.nomes_dos_campos(),
        [
            "hdnInfraTipoPagina",
            "sbmPesquisar",
            "rdoPesquisarEm",
            "txtProtocoloPesquisa",
            "hdnInicio"
        ]
    );
    assert_eq!(
        s.corpo(),
        b"hdnInfraTipoPagina=1&sbmPesquisar=Pesquisar&rdoPesquisarEm=P&txtProtocoloPesquisa=0001&hdnInicio=0"
    );
    assert_eq!(s.efeito(), OPERACAO.efeito);
}

#[test]
fn oculto_fora_do_formulario_nao_participa() {
    let s = derivar_html(&padrao(), &[]).unwrap();
    assert!(
        !s.nomes_dos_campos()
            .iter()
            .any(|n| n == "hdnInfraPrefixoCookie")
    );
}

#[test]
fn controle_desabilitado_nao_e_enviado_como_no_navegador() {
    let html = padrao().replace(
        "</form>",
        "<input type='hidden' name='hdnExtra' value='1' disabled></form>",
    );
    let s = derivar_html(&html, &[]).unwrap();
    assert!(!s.nomes_dos_campos().iter().any(|n| n == "hdnExtra"));
}

#[test]
fn oculto_exigido_omitido_e_recusado() {
    let html = padrao().replace("name='hdnInicio'", "name='hdnOutro'");
    assert_eq!(
        erro(&html, &[]),
        ErroSubmissaoFormulario::CampoOcultoAusenteNoFormulario {
            campo: campo("hdnInicio")
        }
    );
}

#[test]
fn oculto_repetido_nao_escolhe_o_primeiro() {
    let html = padrao().replace(
        "</form>",
        "<input type='hidden' name='hdnInicio' value='50'></form>",
    );
    assert_eq!(
        erro(&html, &[]),
        ErroSubmissaoFormulario::CampoOcultoRepetidoNoFormulario {
            campo: campo("hdnInicio")
        }
    );
}

#[test]
fn campo_inventado_fora_da_lista_e_recusado() {
    assert_eq!(
        erro(&padrao(), &[("selData", "0")]),
        ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: campo("selData")
        }
    );
    assert_eq!(
        erro(&padrao(), &[("hdnInicio", "50")]),
        ErroSubmissaoFormulario::CampoForaDaListaDaOperacao {
            campo: campo("hdnInicio")
        }
    );
}

#[test]
fn campo_da_lista_que_a_tela_nao_publicou_e_recusado() {
    let html = padrao().replace("name='txtProtocoloPesquisa'", "name='txtOutro'");
    assert_eq!(
        erro(&html, &[("txtProtocoloPesquisa", "1")]),
        ErroSubmissaoFormulario::CampoNaoPublicadoPeloFormulario {
            campo: campo("txtProtocoloPesquisa")
        }
    );
}

#[test]
fn campo_informado_duas_vezes_e_recusado() {
    assert_eq!(
        erro(
            &padrao(),
            &[("rdoPesquisarEm", "P"), ("rdoPesquisarEm", "D")]
        ),
        ErroSubmissaoFormulario::CampoInformadoMaisDeUmaVez {
            campo: campo("rdoPesquisarEm")
        }
    );
}

#[test]
fn unidade_da_tela_precisa_ser_a_solicitada() {
    let html = padrao();
    let url = url_servida();
    let r = derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&url, &html),
        "OUTRA",
        &[],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroSubmissaoFormulario::UnidadeDaTelaDiferenteDaSolicitada
    );
    let sem = html.replace("lnkInfraUnidade", "lnkOutro");
    assert_eq!(
        erro(&sem, &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::ausente(
            "identificacao_da_unidade_ausente_na_tela"
        ))
    );
    let repetida = html.replace("<form", "<a id='lnkInfraUnidade'>OUTRA</a><form");
    assert_eq!(
        erro(&repetida, &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::repetido(
            "identificacoes_da_unidade_repetidas_na_tela"
        ))
    );
}

#[test]
fn action_relativo_resolve_contra_a_url_servida_nao_contra_a_raiz() {
    // Relativo à URL servida (/sei/): autorizado.
    assert!(derivar_html(&padrao(), &[]).is_ok());
    // Resolvido contra a raiz do host, o controlador fica fora da base /sei.
    let raiz = action_padrao().replacen("controlador.php", "/controlador.php", 1);
    assert_eq!(
        erro(&com_action(&raiz), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao
    );
}

#[test]
fn action_para_outro_host_ou_caminho_e_recusado() {
    let query = format!("controlador.php?acao=protocolo_pesquisar&amp;infra_hash={HASH_FORM}");
    for action in [
        format!("https://outro.exemplo.gov.br/sei/{query}"),
        format!("//outro.exemplo.gov.br/sei/{query}"),
        format!("/sip/{query}"),
        format!("../{query}"),
        format!("modulos/{query}"),
    ] {
        assert_eq!(
            erro(&com_action(&action), &[]),
            ErroSubmissaoFormulario::AcaoDoFormularioForaDaInstalacao,
            "{action}"
        );
    }
    let http = format!("http://sei.exemplo.gov.br/sei/{query}");
    assert_eq!(
        erro(&com_action(&http), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps
    );
    let js = "javascript:enviar()";
    assert_eq!(
        erro(&com_action(js), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioNaoEhUrlHttps
    );
}

#[test]
fn action_ausente_sem_hash_ou_de_outra_operacao_e_recusado() {
    assert_eq!(
        erro(&pagina("", INTERIOR), &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::ausente("acao_do_formulario_ausente"))
    );
    assert_eq!(
        erro(&com_action("#ancora"), &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::ausente("acao_do_formulario_ausente"))
    );
    let sem_hash = "controlador.php?acao=protocolo_pesquisar";
    assert_eq!(
        erro(&com_action(sem_hash), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico
    );
    let curto = "controlador.php?acao=protocolo_pesquisar&amp;infra_hash=abc";
    assert_eq!(
        erro(&com_action(curto), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioSemInfraHashUnico
    );
    let outra =
        action_padrao().replacen("acao=protocolo_pesquisar", "acao=procedimento_excluir", 1);
    assert_eq!(
        erro(&com_action(&outra), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao
    );
    let dupla = action_padrao().replace("acao_origem=", "acao=");
    assert_eq!(
        erro(&com_action(&dupla), &[]),
        ErroSubmissaoFormulario::AcaoDoFormularioDiferenteDaOperacao
    );
}

#[test]
fn botao_de_envio_ausente_ou_repetido_e_recusado() {
    let sem = padrao().replace(
        "type='submit' id='sbmPesquisar'",
        "type='button' id='sbmPesquisar'",
    );
    assert_eq!(
        erro(&sem, &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::ausente("botao_de_envio_ausente_no_formulario"))
    );
    let dois = padrao().replace(
        "</form>",
        "<input type='submit' name='sbmPesquisar' value='Pesquisar'></form>",
    );
    assert_eq!(
        erro(&dois, &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::repetido(
            "botoes_de_envio_repetidos_no_formulario"
        ))
    );
}

#[test]
fn formulario_ausente_ou_com_identificador_repetido_e_recusado() {
    let ausente = padrao().replace("frmPesquisaProtocolo", "frmOutro");
    assert_eq!(
        erro(&ausente, &[]),
        ErroSubmissaoFormulario::Html(ErroHtml::ausente("formulario_ausente_na_tela"))
    );
    let duplicado = padrao().replace(
        "</html>",
        "<form name='frmPesquisaProtocolo' method='post'></form></html>",
    );
    assert_eq!(
        erro(&duplicado, &[]),
        ErroSubmissaoFormulario::FormulariosComMesmoIdentificadorNaTela
    );
}

#[test]
fn metodo_e_codificacao_nao_observados_sao_recusados() {
    let get = padrao().replace("method='post'", "method='get'");
    assert_eq!(
        erro(&get, &[]),
        ErroSubmissaoFormulario::MetodoDoFormularioDiferenteDePost
    );
    let multipart = padrao().replace(
        "method='post'",
        "method='POST' enctype='multipart/form-data'",
    );
    assert_eq!(
        erro(&multipart, &[]),
        ErroSubmissaoFormulario::EnctypeDoFormularioNaoUrlencoded
    );
    let charset = padrao().replace("method='post'", "method='post' accept-charset='utf-8'");
    assert_eq!(
        erro(&charset, &[]),
        ErroSubmissaoFormulario::AcceptCharsetPublicadoNoFormulario
    );
}

#[test]
fn hdn_acao_publicado_sem_regra_da_operacao_e_recusado() {
    let html = padrao().replace(
        "</form>",
        "<input type='hidden' name='hdnAcao' value=''></form>",
    );
    assert_eq!(
        erro(&html, &[]),
        ErroSubmissaoFormulario::HdnAcaoPublicadoSemRegraDaOperacao
    );
}

#[test]
fn valor_iso_8859_1_e_codificado_pelos_bytes_do_charset() {
    let s = derivar_html(&padrao(), &[("txtProtocoloPesquisa", "ação 1/2+&")]).unwrap();
    let corpo = String::from_utf8(s.corpo().to_vec()).unwrap();
    assert!(
        corpo.contains("txtProtocoloPesquisa=a%E7%E3o+1%2F2%2B%26&"),
        "{corpo}"
    );
    assert_eq!(
        erro(&padrao(), &[("txtProtocoloPesquisa", "€")]),
        ErroSubmissaoFormulario::ValorNaoRepresentavelNaCodificacao {
            campo: campo("txtProtocoloPesquisa")
        }
    );
}

#[test]
fn valor_utf8_usa_bytes_utf8_quando_a_tela_declara_utf8() {
    let html = padrao();
    let url = url_servida();
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao: CodificacaoDaTela::Utf8,
    };
    let campos = [("txtProtocoloPesquisa", "ação 🚀")];
    let s = derivar_submissao(&OPERACAO, &instalacao(), &tela, "UNIDADE TESTE", &campos).unwrap();
    let corpo = String::from_utf8(s.corpo().to_vec()).unwrap();
    assert!(
        corpo.contains("txtProtocoloPesquisa=a%C3%A7%C3%A3o+%F0%9F%9A%80&"),
        "{corpo}"
    );
}

#[test]
fn entidades_html_do_oculto_sao_decodificadas_ou_recusadas() {
    let amp = padrao().replace("value='1'>", "value='a&amp;b&#233;'>");
    let s = derivar_html(&amp, &[]).unwrap();
    assert!(s.corpo().starts_with(b"hdnInfraTipoPagina=a%26b%E9&"));
    let desconhecida = padrao().replace("value='1'>", "value='&inventada;'>");
    assert_eq!(
        erro(&desconhecida, &[]),
        ErroSubmissaoFormulario::EntidadeHtmlDesconhecidaNoAtributo {
            campo: campo("hdnInfraTipoPagina")
        }
    );
}

#[test]
fn efeito_nao_avaliado_recusa_antes_de_ler_a_tela() {
    let op = OperacaoDeFormulario {
        efeito: EfeitoAvaliado::Desconhecido,
        ..OPERACAO
    };
    let url = url_servida();
    let r = derivar_submissao(
        &op,
        &instalacao(),
        &tela_iso(&url, ""),
        "UNIDADE TESTE",
        &[],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroSubmissaoFormulario::EfeitoDaOperacaoNaoAvaliado
    );
}

#[test]
fn efeitos_avaliados_sao_aceitos_e_preservados() {
    let html = padrao();
    let url = url_servida();
    for efeito in [
        EfeitoAvaliado::ConsultaSemEfeitoObservado { evidencia: "teste" },
        EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::UnidadeAtual,
        },
        EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PreferenciaDeVisualizacao,
        },
    ] {
        let op = OperacaoDeFormulario { efeito, ..OPERACAO };
        let s = derivar_submissao(
            &op,
            &instalacao(),
            &tela_iso(&url, &html),
            "UNIDADE TESTE",
            &[],
        );
        assert_eq!(s.unwrap().efeito(), efeito);
    }
}

#[test]
fn tela_fora_da_instalacao_ou_url_invalida_e_recusada() {
    let html = padrao();
    let outra = format!(
        "https://outro.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisar&infra_hash={HASH_TELA}"
    );
    let r = derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(&outra, &html),
        "UNIDADE TESTE",
        &[],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroSubmissaoFormulario::Transporte(FalhaTransporte::OrigemNaoAutorizada)
    );
    let invalida = "http://sei.exemplo.gov.br/sei/controlador.php";
    let r = derivar_submissao(
        &OPERACAO,
        &instalacao(),
        &tela_iso(invalida, &html),
        "UNIDADE TESTE",
        &[],
    );
    assert_eq!(
        r.unwrap_err(),
        ErroSubmissaoFormulario::Transporte(FalhaTransporte::UrlInvalida)
    );
}

#[test]
fn tela_acima_do_limite_e_recusada() {
    let html = format!("{}{}", padrao(), " ".repeat(crate::MAX_HTML_BYTES));
    assert_eq!(
        erro(&html, &[]),
        ErroSubmissaoFormulario::Transporte(FalhaTransporte::RespostaExcessiva)
    );
}

#[test]
fn debug_da_submissao_nao_expoe_hash_referer_nem_valores() {
    let s = derivar_html(&padrao(), &[("txtProtocoloPesquisa", "VALOR-SIGILOSO")]).unwrap();
    let texto = format!("{s:?}");
    for proibido in [
        HASH_FORM,
        HASH_TELA,
        "infra_hash",
        "VALOR-SIGILOSO",
        "Pesquisar&",
    ] {
        assert!(!texto.contains(proibido), "{proibido} em {texto}");
    }
    assert!(texto.contains("txtProtocoloPesquisa"));
    assert!(texto.contains("sei.exemplo.gov.br"));
}
