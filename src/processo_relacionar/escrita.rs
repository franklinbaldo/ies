//! Remover relacionamento: action servida no handler, POST único e releitura.
//! A remoção não é inferida do HTML devolvido pelo POST nem só do estado final.
use super::{FalhaRelacionar as E, tela};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    derivar_submissao_com_action, enviar_submissao_via,
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
            E::EfeitoIncerto(Some(erro))
        } else {
            E::ConsultaDerivacao(erro)
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir
                .localizacao()
                .is_some_and(|loc| aceite_da_remocao(loc, redir.url_do_envio(), id_origem)) => {}
        _ => return Err(E::EfeitoIncerto(None)),
    }
    let (nova_tela, _) = obter_tela(rede, cookie, link).map_err(|_| E::EfeitoIncerto(None))?;
    let linhas = tela::ler_linhas(&nova_tela).map_err(|_| E::EfeitoIncerto(None))?;
    if linhas.iter().any(|linha| linha.id == id_alvo) {
        return Err(E::EfeitoIncerto(None));
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
