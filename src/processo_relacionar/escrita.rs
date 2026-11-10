//! Remover relacionamento: action servida no handler, POST único e releitura.
//! A remoção não é inferida do HTML devolvido pelo POST nem só do estado final.
use super::{FalhaRelacionar as E, tela};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao, derivar_submissao_com_action, enviar_submissao_via,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{Contador, obter_tela};
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::transporte::{conferir_chave_de_consulta, conferir_valor_de_consulta};
use crate::visao_do_controle::EnvioDoControle as _;

const REMOVER: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_excluir_relacionamento",
    formulario: "frmProcedimentoListarRelacionamentos",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraItemId"],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::RelacionamentosDoProcesso,
    },
};

const ADICIONAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "procedimento_relacionar",
    formulario: "frmProcedimentoListarRelacionamentos",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmAdicionar"),
    campos_ocultos_exigidos: &["hdnIdProtocolo"],
    campos_ocultos_reescritos: &["hdnIdProtocolo"],
    campos_usuario: &["txtProtocolo", "txtIdentificacaoProtocolo"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::RelacionamentosDoProcesso,
    },
};

/// A action do script precisa identificar o processo que abriu a tela.
fn action_do_processo(url: &str, id: &str) -> bool {
    let Some(query) = url.strip_prefix("controlador.php?") else {
        return false;
    };
    let mut ids = Vec::new();
    for par in query.split('&') {
        let Some((chave, valor)) = par.split_once('=') else {
            return false;
        };
        if conferir_chave_de_consulta(chave).is_err() || conferir_valor_de_consulta(valor).is_err()
        {
            return false;
        }
        if chave == "id_procedimento" {
            ids.push(valor);
        }
    }
    ids.as_slice() == [id]
}

/// O 302 de aceite volta à tela; resultado=1 e ação únicos.
fn aceite_da_remocao(localizacao: &str, url_post: &str, id: &str) -> bool {
    let Some(query) = consulta_do_controlador(localizacao, url_post) else {
        return false;
    };
    let (mut acoes, mut resultados, mut ids) = (0, 0, 0);
    for par in query.split('&') {
        let Some((chave, valor)) = par.split_once('=') else {
            return false;
        };
        if conferir_chave_de_consulta(chave).is_err() || conferir_valor_de_consulta(valor).is_err()
        {
            return false;
        }
        match chave {
            "acao" if valor == "procedimento_relacionar" => acoes += 1,
            "resultado" if valor == "1" => resultados += 1,
            "id_procedimento" if valor == id => ids += 1,
            "acao" | "resultado" | "id_procedimento" => return false,
            _ => {}
        }
    }
    acoes == 1 && resultados == 1 && ids <= 1
}

/// O sucesso requer duas provas: POST com 302 de aceite e GET novo sem vínculo.
/// Mesmo que a GET já não mostre o vínculo, falha/recusa do POST jamais vira 0.
#[allow(clippy::too_many_arguments)] // Contexto de origem, formulário e transporte explícitos.
pub(super) fn remover<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    id_alvo: &str,
    relacionados_antes: &[tela::Relacionamento],
    rede: &mut Contador<'_, V>,
) -> Result<(), E> {
    let alvo = relacionados_antes
        .iter()
        .find(|relacao| relacao.id == id_alvo)
        .ok_or(E::Estrutura("destino_a_remover_sem_identificador"))?;
    let codificacao = codificacao_da_tela(mime).ok_or(E::Estrutura(
        "charset_da_tela_de_relacionamentos_nao_suportado",
    ))?;
    let servida = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let url_exclusao = tela::url_exclusao(html).map_err(E::Estrutura)?;
    // A action deve pertencer ao mesmo processo de origem, não a outro.
    let id_origem = link
        .split('?')
        .nth(1)
        .and_then(|query| {
            query
                .split('&')
                .find_map(|par| par.strip_prefix("id_procedimento="))
        })
        .ok_or(E::Estrutura("id_da_origem_na_action_ausente"))?;
    if !action_do_processo(&url_exclusao, id_origem) {
        return Err(E::Estrutura("action_de_remocao_de_outro_processo"));
    }
    let submissao = derivar_submissao_com_action(
        &REMOVER,
        instalacao,
        &servida,
        "",
        &[("hdnInfraItemId", id_alvo)],
        Some(&url_exclusao),
    )
    .map_err(E::ConsultaDerivacao)?;
    let mut enviou = false;
    let resultado = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    });
    let resposta = resultado.map_err(|erro| {
        if enviou {
            E::NaoConfirmado(Some(erro))
        } else {
            E::ConsultaDerivacao(erro)
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir
                .localizacao()
                .is_some_and(|loc| aceite_da_remocao(loc, redir.url_do_envio(), id_origem)) => {}
        _ => return Err(E::NaoConfirmado(None)),
    }
    let (nova_tela, _) = obter_tela(rede, cookie, link).map_err(|_| E::NaoConfirmado(None))?;
    let linhas = tela::ler_linhas(&nova_tela).map_err(|_| E::NaoConfirmado(None))?;
    // Um id removido não prova a remoção do NUP: a relação pode ter
    // reaparecido sob outro id, ou outra relação pode ter sido perdida.
    let destino_permanece = linhas
        .iter()
        .any(|linha| linha.id == id_alvo || super::mesmo_protocolo(&linha.nup, &alvo.nup));
    let outras_preservadas = relacionados_antes
        .iter()
        .filter(|relacao| relacao.id != id_alvo)
        .all(|relacao| {
            linhas.iter().any(|linha| {
                linha.id == relacao.id && super::mesmo_protocolo(&linha.nup, &relacao.nup)
            })
        });
    if destino_permanece || !outras_preservadas {
        return Err(E::NaoConfirmado(None));
    }
    Ok(())
}

/**
 * A tela PEN1 publica `sbmAdicionar`, `txtProtocolo`,
 * `txtIdentificacaoProtocolo` e `hdnIdProtocolo` (Todos @323eb01).
 * Um POST, aceite 302 explícito, releitura independente. Sem retry.
 */
#[allow(clippy::too_many_arguments)] // Contexto observado do formulário, identidade e rede.
pub(super) fn adicionar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    id_destino: &str,
    nup_destino: &str,
    tipo_destino: &str,
    relacionados_antes: &[tela::Relacionamento],
    rede: &mut Contador<'_, V>,
) -> Result<(), E> {
    let codificacao = codificacao_da_tela(mime).ok_or(E::Estrutura(
        "charset_da_tela_de_relacionamentos_nao_suportado",
    ))?;
    let servida = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &ADICIONAR,
        instalacao,
        &servida,
        "",
        &[
            ("txtProtocolo", nup_destino),
            ("txtIdentificacaoProtocolo", tipo_destino),
            ("hdnIdProtocolo", id_destino),
        ],
    )
    .map_err(E::ConsultaDerivacao)?;
    let id_origem = link
        .split('?')
        .nth(1)
        .and_then(|query| {
            query
                .split('&')
                .find_map(|par| par.strip_prefix("id_procedimento="))
        })
        .ok_or(E::Estrutura("id_da_origem_na_action_ausente"))?;
    let mut enviou = false;
    let resultado = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    });
    let resposta = resultado.map_err(|erro| {
        if enviou {
            E::NaoConfirmado(Some(erro))
        } else {
            E::ConsultaDerivacao(erro)
        }
    })?;
    match resposta {
        // O mesmo marcador de retorno observado na tela de relacionamento:
        // ação única, resultado=1 e, quando presente, origem identificada.
        RespostaDoEnvio::Redirecionamento(redir)
            if redir
                .localizacao()
                .is_some_and(|loc| aceite_da_remocao(loc, redir.url_do_envio(), id_origem)) => {}
        _ => return Err(E::NaoConfirmado(None)),
    }
    let (nova_tela, _) = obter_tela(rede, cookie, link).map_err(|_| E::NaoConfirmado(None))?;
    let linhas = tela::ler_linhas(&nova_tela).map_err(|_| E::NaoConfirmado(None))?;
    let destino_confirmado = linhas
        .iter()
        .any(|linha| linha.id == id_destino && super::mesmo_protocolo(&linha.nup, nup_destino));
    let anteriores_preservados = relacionados_antes.iter().all(|anterior| {
        linhas.iter().any(|atual| {
            atual.id == anterior.id && super::mesmo_protocolo(&atual.nup, &anterior.nup)
        })
    });
    if !destino_confirmado || !anteriores_preservados {
        return Err(E::NaoConfirmado(None));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aceite_precisa_da_acao_e_resultado_uma_vez() {
        let url = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_excluir_relacionamento";
        assert!(aceite_da_remocao(
            "controlador.php?acao=procedimento_relacionar&resultado=1",
            url,
            "10"
        ));
        assert!(!aceite_da_remocao(
            "controlador.php?acao=procedimento_relacionar&resultado=0",
            url,
            "10"
        ));
        assert!(!aceite_da_remocao(
            "controlador.php?acao=procedimento_relacionar&resultado=1&resultado=1",
            url,
            "10"
        ));
        assert!(!aceite_da_remocao(
            "https://host-estranho.example/controlador.php?acao=procedimento_relacionar&resultado=1",
            url,
            "10"
        ));
    }
}
