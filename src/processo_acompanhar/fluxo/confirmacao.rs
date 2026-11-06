//! Conferência independente das escritas de Acompanhamento Especial (#254).
//! Um POST no máximo, sem retry; falha incerta exige leitura independente.
use super::super::conferir_inclusao;
use super::*;
use crate::formulario_web::{ErroSubmissaoFormulario, RespostaDoEnvio, enviar_submissao_via};

/// Observação finita para diagnosticar o aceite sem revelar o POST ou Location.
fn classificar_resposta_post(
    resposta: &Result<RespostaDoEnvio, ErroSubmissaoFormulario>,
) -> RespostaPost {
    match resposta {
        Ok(RespostaDoEnvio::Html(_)) => RespostaPost::HtmlSemRedirecionamento,
        Ok(RespostaDoEnvio::Redirecionamento(_)) => RespostaPost::RedirecionamentoObservado,
        Err(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(_)) => {
            RespostaPost::ValidacaoInfraRecusada
        }
        Err(ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede)) => {
            RespostaPost::RedeSemResposta
        }
        Err(erro) if erro.status_http().is_some() => RespostaPost::StatusHttpRecusado,
        Err(_) => RespostaPost::FalhaDaSubmissao,
    }
}

/// Consome uma única submissão e verifica o resultado por leitura nova.
/// Mesmo um erro de transporte no POST pode ter produzido efeito.
pub(super) fn confirmar_inclusao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    contexto: &ContextoConferencia<'_>,
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
    // A releitura prova o efeito, mas não legitima um 302 para outra
    // operação ou outro processo. Sem resposta de rede, o POST ainda
    // pode ter sido consumido: apenas uma leitura exata pode conciliá-lo.
    let aceite = matches!(
        &resposta,
        Err(crate::formulario_web::ErroSubmissaoFormulario::Transporte(
            FalhaTransporte::Rede
        ))
    ) || aceite_do_gerenciador(&resposta, contexto.instalacao, contexto.id);
    let depois = rede.get_sem_seguir(contexto.link, contexto.cookie);
    let depois = match depois {
        Ok(resposta) if resposta.resposta.status_code != 302 => {
            decodificar_resposta(resposta.resposta)
                .ok()
                .and_then(|html| acompanhamentos_da_lista(&html).ok())
        }
        _ => None,
    };
    let verificado = depois.as_deref().is_some_and(|linhas| {
        conferir_inclusao(contexto.antes, linhas, &plano.grupo, &plano.observacao).is_ok()
    });
    if !aceite || !verificado {
        let motivo = if !aceite {
            MotivoEfeitoIncerto::RespostaSemAceite
        } else if depois.is_none() {
            MotivoEfeitoIncerto::ReleituraIndisponivel
        } else {
            MotivoEfeitoIncerto::EstadoDivergente
        };
        return Err(ErroPrevia::EfeitoIncerto {
            requisicoes: rede.feitas(),
            motivo,
            resposta_post: classificar_resposta_post(&resposta),
        });
    }
    // Somente depois da prova exata podemos publicar o ID observado.
    let novo = depois
        .as_deref()
        .and_then(|linhas| {
            linhas.iter().find(|linha| {
                !contexto
                    .antes
                    .iter()
                    .any(|anterior| anterior.id == linha.id)
                    && linha.grupo == plano.grupo
                    && linha.observacao == plano.observacao
            })
        })
        .ok_or(ErroPrevia::EfeitoIncerto {
            requisicoes: rede.feitas(),
            motivo: MotivoEfeitoIncerto::EstadoDivergente,
            resposta_post: classificar_resposta_post(&resposta),
        })?;
    plano.id_de_remocao = Some(novo.id.clone());
    plano.alterado = true;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}

/// O 302 do POST precisa apontar ao gerenciador do mesmo processo.
/// Usado na inclusão e retirada; nenhuma resposta é seguida automaticamente.
fn aceite_do_gerenciador(
    resposta: &Result<RespostaDoEnvio, crate::formulario_web::ErroSubmissaoFormulario>,
    instalacao: &Instalacao,
    id: &str,
) -> bool {
    let Ok(RespostaDoEnvio::Redirecionamento(redirecionamento)) = resposta else {
        return false;
    };
    let Some(consulta) = redirecionamento.localizacao().and_then(|loc| {
        crate::formulario_web::consulta_do_controlador(loc, redirecionamento.url_do_envio())
    }) else {
        return false;
    };
    crate::arvore_processo::conferir_endereco_relativo(
        instalacao,
        &format!("controlador.php?{consulta}"),
        "acompanhamento_gerenciar",
        id,
    )
    .is_ok()
}

/// A última retirada pode tornar o gerenciador um 302 para o cadastro.
/// Só o cadastro válido do processo, lido por GET explícito, comprova
/// a ausência de linhas; HTML truncado ou redirect estranho são incertos.
fn releitura_da_remocao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    instalacao: &Instalacao,
    id: &str,
    link: &str,
    cookie: &str,
) -> Option<Vec<Acompanhamento>> {
    let res = rede.get_sem_seguir(link, cookie).ok()?;
    if res.resposta.status_code == 302 {
        let url =
            redirecionamento_do_gerenciar(instalacao, res.localizacao.as_deref(), link, id).ok()?;
        let (html, _) = html_da_tela(rede, cookie, &url).ok()?;
        grupos_do_cadastro(&html, id).ok()?;
        return Some(Vec::new());
    }
    let html = decodificar_resposta(res.resposta).ok()?;
    acompanhamentos_da_lista(&html).ok()
}

/// Elementos observados na mesma sessão e abertura que originaram o
/// formulário da remoção. O POST é consumido uma vez.
pub(super) struct ContextoConferencia<'a> {
    pub(super) instalacao: &'a Instalacao,
    pub(super) id: &'a str,
    pub(super) link: &'a str,
    pub(super) cookie: &'a str,
    pub(super) antes: &'a [Acompanhamento],
}

/// A escrita nunca se repete, nem após erro de rede. O único critério de
/// sucesso é o aceite observado E a leitura nova com desaparecimento
/// exclusivo da linha alvo, preservando todas as demais identidades.
pub(super) fn confirmar_remocao<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    sessao: &SessaoDoProcesso<'_>,
    contexto: &ContextoConferencia<'_>,
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
    let aceite = aceite_do_gerenciador(&resposta, contexto.instalacao, contexto.id);
    let depois = releitura_da_remocao(
        rede,
        contexto.instalacao,
        contexto.id,
        contexto.link,
        contexto.cookie,
    );
    let verificado = depois.as_deref().is_some_and(|linhas| {
        super::super::conferir_remocao(
            contexto.antes,
            linhas,
            plano.id_de_remocao.as_deref().unwrap_or_default(),
        )
        .is_ok()
    });
    if !aceite || !verificado {
        let motivo = if !aceite {
            MotivoEfeitoIncerto::RespostaSemAceite
        } else if depois.is_none() {
            MotivoEfeitoIncerto::ReleituraIndisponivel
        } else {
            MotivoEfeitoIncerto::EstadoDivergente
        };
        return Err(ErroPrevia::EfeitoIncerto {
            requisicoes: rede.feitas(),
            motivo,
            resposta_post: classificar_resposta_post(&resposta),
        });
    }
    plano.alterado = true;
    plano.requisicoes = rede.feitas();
    Ok(plano)
}
