//! Um único POST de anexação mais releitura independente da lista observada.
//! O aceite 302 sozinho nunca demonstra que houve anexação.
use super::{Destino, EtapaNaoComprovada, FalhaAnexar, ler_tela};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{Contador, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::visao_do_controle::EnvioDoControle as _;

const ANEXAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::ProcedimentoAnexar,
    formulario: "frmProcedimentoAnexar",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmAnexar"),
    campos_ocultos_exigidos: &["hdnIdProtocolo", "hdnAnexacao"],
    campos_ocultos_reescritos: &["hdnIdProtocolo", "hdnAnexacao"],
    campos_usuario: &["txtProtocolo", "txtIdentificacaoProtocolo"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AnexacaoDeProcesso,
    },
};

pub(crate) fn aceite(localizacao: &str, url_post: &str, principal: &str) -> bool {
    let Some(query) = consulta_do_controlador(localizacao, url_post) else {
        return false;
    };
    ConsultaDoControlador::ler(query).is_ok_and(|consulta| {
        consulta.acao() == Ok(AcaoDoSei::ProcedimentoAnexar.como_na_url())
            && consulta.unico("resultado") == Ok("1")
            && consulta.unico("id_procedimento") == Ok(principal)
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "Dados da única tela, origem e transporte"
)]
pub(super) fn confirmar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    html: &crate::tela_html::HtmlServido,
    mime: &str,
    id_principal: &str,
    destino: &Destino,
    assinado: bool,
    anteriores: &[String],
    rede: &mut Contador<'_, V>,
) -> Result<(), FalhaAnexar> {
    let codificacao =
        codificacao_da_tela(mime).ok_or(FalhaAnexar::CharsetDaTelaDeAnexacaoNaoSuportado)?;
    let tela = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &ANEXAR,
        instalacao,
        &tela,
        None,
        &[
            ("txtProtocolo", &destino.nup),
            ("txtIdentificacaoProtocolo", &destino.tipo),
            ("hdnIdProtocolo", &destino.id),
            ("hdnAnexacao", if assinado { "S" } else { "N" }),
        ],
    )
    .map_err(FalhaAnexar::ConsultaDerivacao)?;
    let mut enviou = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|e| {
        if enviou {
            FalhaAnexar::NaoConfirmado(EtapaNaoComprovada::RespostaDoPost)
        } else {
            FalhaAnexar::ConsultaDerivacao(e)
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(r)
            if r.localizacao()
                .is_some_and(|loc| aceite(loc, r.url_do_envio(), id_principal)) => {}
        RespostaDoEnvio::Html(_) | RespostaDoEnvio::Redirecionamento(_) => {
            return Err(FalhaAnexar::NaoConfirmado(
                EtapaNaoComprovada::RespostaDoPost,
            ));
        }
    }
    let (depois, _) =
        obter_tela(rede, cookie, link).map_err(FalhaAnexar::NaoConfirmadoNaListaAposPost)?;
    let lista = ler_tela(&depois)
        .map_err(FalhaAnexar::NaoConfirmadoNaTelaDaListaAposPost)?
        .anexados;
    if !lista.contains(&destino.id) || !anteriores.iter().all(|id| lista.contains(id)) {
        return Err(FalhaAnexar::NaoConfirmado(
            EtapaNaoComprovada::ListaAposPost,
        ));
    }
    Ok(())
}
