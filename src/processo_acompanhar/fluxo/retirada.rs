//! A retirada confirmada consome um POST e exige releitura independente.
use super::*;

/// Depois do único POST, comprovar a perda exclusiva da linha escolhida.
/// Resposta do POST ou releitura insuficiente devolvem efeito incerto (saída 3).
#[allow(clippy::too_many_arguments)] // Contexto exato da mesma leitura e submissão.
pub(super) fn confirmar_remocao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    instalacao: &Instalacao,
    id: &str,
    link: &str,
    cookie: &str,
    antes: &[Acompanhamento],
    mut plano: Previa,
    submissao: SubmissaoDerivada,
) -> Result<Previa, ErroPrevia> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |formulario, cookie| {
        enviado = true;
        rede.post(formulario, cookie)
    });
    if !enviado {
        return Err(ErroPrevia::Formulario(
            resposta
                .err()
                .ok_or(ErroHtml::valor("tela_de_acompanhamento_inesperada"))?,
        ));
    }
    let aceite = match &resposta {
        Ok(RespostaDoEnvio::Redirecionamento(redirecionamento)) => redirecionamento
            .localizacao()
            .and_then(|local| {
                crate::formulario_web::consulta_do_controlador(
                    local,
                    redirecionamento.url_do_envio(),
                )
            })
            .is_some_and(|consulta| {
                crate::arvore_processo::conferir_endereco_relativo(
                    instalacao,
                    &format!("controlador.php?{consulta}"),
                    "acompanhamento_gerenciar",
                    id,
                )
                .is_ok()
            }),
        _ => false,
    };
    let depois = rede.get_sem_seguir(link, cookie).ok().and_then(|res| {
        if res.resposta.status_code == 302 {
            // Sem comprovação da última linha removida, nunca afirmar êxito.
            return None;
        }
        decodificar_resposta(res.resposta)
            .ok()
            .and_then(|html| acompanhamentos_da_lista(&html).ok())
    });
    let confirmado = aceite
        && depois.as_deref().is_some_and(|linhas| {
            super::conferir_remocao(
                antes,
                linhas,
                plano.id_de_remocao.as_deref().unwrap_or_default(),
            )
            .is_ok()
        });
    if !confirmado {
        return Err(ErroPrevia::EfeitoIncerto);
    }
    plano.estado = EstadoDoAcompanhamento::Removido;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}
