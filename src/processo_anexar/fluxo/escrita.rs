//! Um único POST de anexação mais releitura independente da lista observada.
//! O aceite 302 sozinho nunca demonstra que houve anexação.
use super::{Destino, EtapaNaoComprovada, FalhaAnexar as E, ler_tela};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{Contador, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
use crate::visao_do_controle::EnvioDoControle as _;

const ANEXAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_anexar",
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

fn aceite(localizacao: &str, url_post: &str, principal: &str) -> bool {
    let Some(query) = consulta_do_controlador(localizacao, url_post) else {
        return false;
    };
    let (mut acao, mut resultado, mut id) = (0, 0, 0);
    for parte in query.split('&') {
        let Some((chave, valor)) = parte.split_once('=') else {
            return false;
        };
        if conferir_chave_de_consulta(chave).is_err() || conferir_valor_de_consulta(valor).is_err()
        {
            return false;
        }
        match chave {
            "acao" if valor == "procedimento_anexar" => acao += 1,
            "resultado" if valor == "1" => resultado += 1,
            "id_procedimento" if valor == principal => id += 1,
            "acao" | "resultado" | "id_procedimento" => return false,
            _ => {}
        }
    }
    acao == 1 && resultado == 1 && id == 1
}

#[allow(clippy::too_many_arguments)] // Dados da única tela, origem e transporte.
pub(super) fn confirmar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    id_principal: &str,
    destino: &Destino,
    assinado: bool,
    anteriores: &[String],
    rede: &mut Contador<'_, V>,
) -> Result<(), E> {
    let codificacao = codificacao_da_tela(mime)
        .ok_or(E::Estrutura("charset_da_tela_de_anexacao_nao_suportado"))?;
    let tela = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &ANEXAR,
        instalacao,
        &tela,
        "",
        &[
            ("txtProtocolo", &destino.nup),
            ("txtIdentificacaoProtocolo", &destino.tipo),
            ("hdnIdProtocolo", &destino.id),
            ("hdnAnexacao", if assinado { "S" } else { "N" }),
        ],
    )
    .map_err(E::ConsultaDerivacao)?;
    let mut enviou = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|e| {
        if enviou {
            E::NaoConfirmado(EtapaNaoComprovada::RespostaDoPost)
        } else {
            E::ConsultaDerivacao(e)
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(r)
            if r.localizacao()
                .is_some_and(|loc| aceite(loc, r.url_do_envio(), id_principal)) => {}
        _ => return Err(E::NaoConfirmado(EtapaNaoComprovada::RespostaDoPost)),
    }
    let (depois, _) = obter_tela(rede, cookie, link)
        .map_err(|_| E::NaoConfirmado(EtapaNaoComprovada::ListaAposPost))?;
    let lista = ler_tela(&depois)
        .map_err(|_| E::NaoConfirmado(EtapaNaoComprovada::ListaAposPost))?
        .anexados;
    if !lista.contains(&destino.id) || !anteriores.iter().all(|id| lista.contains(id)) {
        return Err(E::NaoConfirmado(EtapaNaoComprovada::ListaAposPost));
    }
    Ok(())
}
