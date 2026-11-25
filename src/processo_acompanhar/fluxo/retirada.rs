//! A retirada confirmada consome um POST e exige releitura independente.
use super::*;
use crate::acao_do_sei::AcaoDoSei;

/// Depois do único POST, comprovar a perda exclusiva da linha escolhida.
/// Resposta do POST ou releitura insuficiente devolvem escrita não confirmada (saída 1).
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
) -> Result<Previa, ErroAcompanhar> {
    let mut enviado = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |formulario, cookie| {
        enviado = true;
        rede.post(formulario, cookie)
    });
    if !enviado {
        return Err(ErroAcompanhar::Formulario(
            resposta
                .err()
                .ok_or(ErroAcompanhar::TelaDeAcompanhamentoInesperada)?,
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
                    AcaoDoSei::AcompanhamentoGerenciar,
                    id,
                )
                .is_ok()
            }),
        _ => false,
    };
    // A releitura é obrigatória mesmo que o POST tenha falhado; não repetir a escrita.
    let resposta = rede
        .get_sem_seguir(link, cookie)
        .map_err(ErroAcompanhar::ReleituraDaRetiradaFalhou)?;
    if resposta.resposta.status_code == 302 {
        // 302 não comprova a ausência exata da linha removida.
        return Err(ErroAcompanhar::RedirecionamentoDeAcompanhamentoInesperado);
    }
    let html = decodificar_resposta(resposta.resposta)
        .map_err(ErroAcompanhar::ReleituraDaRetiradaFalhou)?;
    let depois = acompanhamentos_da_lista(&html)?;
    let confirmado = aceite
        && super::super::conferir_remocao(
            antes,
            &depois,
            plano.id_de_remocao.as_deref().unwrap_or_default(),
        )
        .is_ok();
    if !confirmado {
        return Err(ErroAcompanhar::AcompanhamentoNaoConfirmado);
    }
    plano.estado = EstadoDoAcompanhamento::Removido;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}
