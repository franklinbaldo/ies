//! CLI Web de controle de prazo: leitura, prévia e confirmação local testável.
//! O binário de distribuição continua recusando escrita antes da rede.
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDoProcesso, OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::EnvioDoProcesso;
use crate::processo_obter::{ProcessoAberto, SessaoDoProcesso, abrir_para_escrever};
use crate::processo_prazo::{
    EstadoDoPrazo, PedidoDePrazo, TelaDoPrazo, campos_do_pedido, interpretar_pedido, ler_tela,
};
use crate::visao_do_controle::{EnvioDoControle as _, EnvioHttps};
use crate::{Cli, Domain, Processo, ProcessoAcao, processo_concluir_cli, saida_erro};
use serde_json::{Value, json};
use std::process::ExitCode;

mod exclusao;

pub(crate) fn tentar_prazo(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        acao:
            Some(ProcessoAcao::Prazo {
                numero,
                ate,
                dias,
                excluir,
                confirmar,
            }),
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Ok(nup) = NumeroProcesso::novo(numero) else {
        return Some(uso(cli.json));
    };
    let Ok(pedido) = interpretar_pedido(ate.as_deref(), *dias, *excluir) else {
        return Some(uso(cli.json));
    };
    // Sem --ate, --dias ou --excluir não há o que gravar: uso, antes da rede.
    if *confirmar && pedido == PedidoDePrazo::Ler {
        return Some(uso(cli.json));
    }
    Some(processo_concluir_cli::com_sessao(cli, |sessao, indice| {
        // Aprovada em treinamento (10/11/2026): grava também em produção.
        if *confirmar
            && let Err(causa) = crate::origem_de_treinamento::exigir(
                crate::origem_de_treinamento::Escrita::ProcessoPrazo,
                sessao.origem,
            )
        {
            return crate::origem_de_treinamento::recusa(cli.json, causa);
        }
        if pedido == PedidoDePrazo::Excluir {
            exclusao::executar(cli, &nup, *confirmar, sessao, indice)
        } else {
            previsualizar(cli, &nup, &pedido, *confirmar, sessao, indice)
        }
    }))
}

fn uso(json_mode: bool) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "erro": "parâmetros de prazo inválidos",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: parâmetros de prazo inválidos");
    }
    ExitCode::from(2)
}

fn recusar(json_mode: bool, mensagem: &str) -> ExitCode {
    saida_erro::erro_fonte_http(json_mode, mensagem, None)
}

/// Preserva as causas do GET de formulário sem duplicar tratamento na CLI.
fn consultar_formulario<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    json_mode: bool,
) -> Result<(String, String), ExitCode> {
    obter_tela(rede, cookie, link).map_err(|falha| match falha {
        FalhaDaTela::Transporte(erro) => {
            saida_erro::erro_fonte_http(json_mode, erro.mensagem(), erro.status_http())
        }
        FalhaDaTela::Redirecionada => {
            recusar(json_mode, "a tela de prazo redirecionou inesperadamente")
        }
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            recusar(json_mode, "a tela de prazo devolveu o login")
        }
    })
}

fn previsualizar(
    cli: &Cli,
    nup: &NumeroProcesso,
    pedido: &PedidoDePrazo,
    confirmar: bool,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
) -> ExitCode {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let abertura = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = match abrir_para_escrever(sessao, &abertura, indice, &mut rede) {
        Ok(aberto) => aberto,
        Err(erro) => {
            return saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http());
        }
    };
    let link = match acoes.prazo {
        Ok(Some(link)) => link,
        Ok(None) => {
            return recusar(
                cli.json,
                "a barra do processo não oferece controle de prazo",
            );
        }
        Err(_) => {
            return recusar(
                cli.json,
                "o link de controle de prazo não corresponde ao processo ou à forma observada",
            );
        }
    };
    let cookie = match ler_cookie(sessao) {
        Ok(cookie) => cookie,
        Err(erro) => {
            return saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http());
        }
    };
    let (html, mime) = match consultar_formulario(&mut rede, &cookie, &link, cli.json) {
        Ok(tela) => tela,
        Err(erro) => return erro,
    };
    let Ok(tela) = ler_tela(&html, &aberto.arvore.processo.id) else {
        return recusar(cli.json, "tela de prazo fora da forma observada");
    };
    let tela = if confirmar {
        match confirmar_prazo(
            sessao,
            &cookie,
            &link,
            &html,
            &mime,
            pedido,
            &aberto.arvore.processo.id,
            &mut rede,
        ) {
            Ok(verificada) => verificada,
            Err(FalhaConfirmacao::Preparo) => {
                return recusar(
                    cli.json,
                    "formulário de prazo não permite confirmação segura",
                );
            }
            Err(FalhaConfirmacao::Formulario) => {
                return recusar(cli.json, "submissão de prazo recusada");
            }
            Err(FalhaConfirmacao::NaoConfirmado(causa)) => {
                return nao_confirmado(cli.json, causa.as_ref());
            }
        }
    } else {
        tela
    };
    publicar_previa(cli, nup, pedido, &aberto, &tela, rede.feitas(), confirmar)
}

/// Publica a prévia lida, sem escrever: estado atual, pedido e campos.
fn publicar_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    pedido: &PedidoDePrazo,
    aberto: &ProcessoAberto,
    tela: &TelaDoPrazo,
    requisicoes: u8,
    confirmado: bool,
) -> ExitCode {
    let atual = match &tela.estado {
        EstadoDoPrazo::SemPrazo => json!({"tipo": "sem_prazo"}),
        EstadoDoPrazo::Data(data) => json!({"tipo": "data", "data": data}),
        EstadoDoPrazo::Dias(dias) => json!({"tipo": "dias", "dias": dias}),
        EstadoDoPrazo::Concluido => json!({"tipo": "concluido"}),
    };
    let solicitado = match pedido {
        PedidoDePrazo::Ate(data) => json!({"ate": data}),
        PedidoDePrazo::Dias(dias) => json!({"dias": dias}),
        PedidoDePrazo::Ler | PedidoDePrazo::Excluir => Value::Null,
    };
    let campos: Vec<_> = campos_do_pedido(pedido)
        .into_iter()
        .map(|(nome, _)| nome)
        .collect();
    let resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "unidade": aberto.unidade,
        "origem": "web",
        "atual": atual,
        "valor_do_checkbox_de_dias_uteis": tela.valor_dos_uteis,
        "solicitado": solicitado,
        "campos_da_previa": campos,
        "confirmado": confirmado,
        "requisicoes": requisicoes,
    });
    if cli.json {
        println!("{resultado}");
    } else {
        println!("Processo {} — prazo atual: {}", nup.as_str(), atual);
        if confirmado {
            println!("Prazo confirmado por releitura: {solicitado}.");
        } else if !solicitado.is_null() {
            println!("Prévia: {solicitado}. Confirmação disponível somente no simulador local.");
        }
        println!("Requisições: {requisicoes}");
    }
    ExitCode::SUCCESS
}

const DEFINIR_PRAZO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "controle_prazo_definir",
    formulario: "frmControlePrazoCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmDefinirControlePrazo"),
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnIdControlePrazo",
        "hdnIdProcedimentos",
    ],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["rdoPrazo", "txtPrazo", "txtDias"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ControleDePrazo,
    },
};

enum FalhaConfirmacao {
    Preparo,
    Formulario,
    NaoConfirmado(Option<ErroSubmissaoFormulario>),
}

/// O 302 deve nomear uma única ação, o resultado e o mesmo processo.
fn aceite_confirmacao(local: &str, url_post: &str, id: &str) -> bool {
    let Some(consulta) = consulta_do_controlador(local, url_post) else {
        return false;
    };
    let mut acoes = Vec::new();
    let mut resultados = Vec::new();
    let mut processos = Vec::new();
    for parte in consulta.split('&') {
        let Some((chave, valor)) = parte.split_once('=') else {
            return false;
        };
        match chave {
            "acao" => acoes.push(valor),
            "resultado" => resultados.push(valor),
            "id_procedimento" => processos.push(valor),
            _ => {}
        }
    }
    acoes == ["controle_prazo_definir"]
        && resultados == ["1"]
        && (processos.is_empty() || processos == [id])
}

/// Apenas o primeiro POST tem efeito; erro depois do envio é incerto.
#[allow(clippy::too_many_arguments)]
fn confirmar_prazo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    cookie: &str,
    link: &str,
    html: &str,
    mime: &str,
    pedido: &PedidoDePrazo,
    id: &str,
    rede: &mut Contador<'_, V>,
) -> Result<TelaDoPrazo, FalhaConfirmacao> {
    if matches!(pedido, PedidoDePrazo::Ler | PedidoDePrazo::Excluir) {
        return Err(FalhaConfirmacao::Preparo);
    }
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(FalhaConfirmacao::Preparo)?;
    let codificacao = codificacao_da_tela(mime).ok_or(FalhaConfirmacao::Preparo)?;
    let servida = TelaServida {
        url: link,
        html,
        codificacao,
    };
    let campos = campos_do_pedido(pedido);
    let valores: Vec<_> = campos
        .iter()
        .map(|(nome, valor)| (*nome, valor.as_str()))
        .collect();
    let submissao = derivar_submissao(&DEFINIR_PRAZO, &instalacao, &servida, "", &valores)
        .map_err(|_| FalhaConfirmacao::Formulario)?;
    let mut enviou = false;
    let resultado = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|erro| {
        if enviou {
            FalhaConfirmacao::NaoConfirmado(Some(erro))
        } else {
            FalhaConfirmacao::Formulario
        }
    })?;
    match resultado {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir
                .localizacao()
                .is_some_and(|local| aceite_confirmacao(local, redir.url_do_envio(), id)) => {}
        // A `UnB` responde 200 com o formulário redesenhado (cassette
        // processo-prazo-unb, interações 12 e 19; live de 10/11/2026): a
        // prova fica com a releitura independente abaixo.
        RespostaDoEnvio::Html(_) => {}
        RespostaDoEnvio::Redirecionamento(_) => return Err(FalhaConfirmacao::NaoConfirmado(None)),
    }
    let (nova, _) =
        obter_tela(rede, cookie, link).map_err(|_| FalhaConfirmacao::NaoConfirmado(None))?;
    let verificada = ler_tela(&nova, id).map_err(|_| FalhaConfirmacao::NaoConfirmado(None))?;
    let hoje = (time::OffsetDateTime::now_utc().date() - time::OffsetDateTime::UNIX_EPOCH.date())
        .whole_days();
    if !crate::processo_prazo::releitura_confirma(pedido, &verificada.estado, hoje) {
        return Err(FalhaConfirmacao::NaoConfirmado(None));
    }
    Ok(verificada)
}

fn nao_confirmado(json_mode: bool, causa: Option<&ErroSubmissaoFormulario>) -> ExitCode {
    let mensagem =
        "O POST pode ter produzido efeito, mas a releitura não confirmou; não repita a operação";
    if json_mode {
        let mut resposta = json!({
            "codigo": "nao_confirmado",
            "erro": mensagem,
            "retentavel": false
        });
        if let Some(status) = causa.and_then(|c| c.status_http()) {
            resposta["status_http"] = json!(status.status);
            if let Some(texto) = &status.mensagem_do_servidor {
                resposta["mensagem_servidor"] = json!(texto);
            }
        }
        println!("{resposta}");
    } else {
        eprintln!("IES: {mensagem}");
    }
    ExitCode::from(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processo_prazo::ErroPrazo;

    #[test]
    fn tipos_de_entrada_distintos_nao_sao_tratados_como_escrita() {
        assert_eq!(
            interpretar_pedido(None, None, false),
            Ok(PedidoDePrazo::Ler)
        );
        assert_eq!(
            interpretar_pedido(None, None, true),
            Ok(PedidoDePrazo::Excluir)
        );
        assert_eq!(
            interpretar_pedido(Some("31/02/2027"), None, false),
            Err(ErroPrazo::DataInvalida)
        );
    }
}
