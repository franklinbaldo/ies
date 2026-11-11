//! BDD-255-02: uma única remoção, derivada do formulário efetivamente servido.
//! O GET de gerenciamento não autoriza POST sem handler, hidden e ação assinada.
use super::{Decisao, ErroMarcar, FalhaDaMarcacao, Operacao, Rotulo, decidir, recusa};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, Redirecionamento302, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao_com_action, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever, obter_processo,
};
use crate::visao_do_controle::EnvioDoControle as _;
use serde_json::json;

const REMOVER: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::AndamentoMarcadorRemover,
    formulario: "frmGerenciarMarcador",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &[
        "hdnInfraTipoPagina",
        "hdnInfraNroItens",
        "hdnInfraItemId",
        "hdnInfraItens",
        "hdnInfraItensHash",
    ],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::MarcadoresNoProcesso,
    },
};

/// Extrai SOMENTE a URL literal assinada atribuída pelo handler da mesma tela.
/// Não executa JS, não troca host, não cria destino a partir de padrões.
fn action_de_remocao(html: &str, id: &str) -> Option<String> {
    let script = html.split_once("function acaoRemover(id,desc)")?.1;
    let inicio = "document.getElementById('frmGerenciarMarcador').action='";
    let url = script.split_once(inicio)?.1.split_once('\'')?.0;
    if !html.contains(&format!("acaoRemover('{id}',")) {
        return None;
    }
    if url.is_empty() || script.matches(inicio).count() != 1 {
        return None;
    }
    Some(url.to_owned())
}

fn aceite_da_remocao(redir: &Redirecionamento302, id: &str) -> bool {
    let Some(consulta) = redir
        .localizacao()
        .and_then(|url| consulta_do_controlador(url, redir.url_do_envio()))
    else {
        return false;
    };
    let (mut acao, mut procedimento, mut origem) = (None, None, None);
    for item in consulta.split('&') {
        let Some((nome, valor)) = item.split_once('=') else {
            return false;
        };
        let destino = match nome {
            "acao" => &mut acao,
            "id_procedimento" => &mut procedimento,
            "acao_origem" => &mut origem,
            _ => continue,
        };
        if destino.replace(valor).is_some() {
            return false;
        }
    }
    acao.and_then(AcaoDoSei::ler) == Some(AcaoDoSei::AndamentoMarcadorGerenciar)
        && origem.and_then(AcaoDoSei::ler) == Some(AcaoDoSei::AndamentoMarcadorRemover)
        && procedimento == Some(id)
}

fn saida_sem_alteracao(
    cli: &Cli,
    nup: &NumeroProcesso,
    processo_id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    requisicoes: u8,
) {
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(),
                "id_procedimento": processo_id,
                "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "acao": "retirar",
                "confirmado": false,
                "alterado": false,
                "nada_a_fazer": true,
                "retirada_possivel": false,
                "requisicoes": requisicoes
            })
        );
    } else {
        println!(
            "Processo {}: nada a fazer com {} (retirar). Requisições: {}.",
            nup.as_str(),
            rotulo.id,
            requisicoes
        );
    }
}

fn saida_confirmada(
    cli: &Cli,
    nup: &NumeroProcesso,
    processo_id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    requisicoes: u8,
) {
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(),
                "id_procedimento": processo_id,
                "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "acao": "retirar",
                "origem": "web",
                "confirmado": true,
                "alterado": true,
                "nada_a_fazer": false,
                "retirada_possivel": true,
                "requisicoes": requisicoes
            })
        );
    } else {
        println!(
            "Remoção confirmada: marcador {}#{} do processo {} (unidade {}). Requisições: {}.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            requisicoes
        );
    }
}

/// Prepara a única submissão a partir da tela servida, sem enviar POST.
/// Recusas aqui preservam o orçamento observado e não geram efeito.
fn preparar_submissao<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    link: &str,
    unidade: &str,
    rotulo: &Rotulo,
) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {
    let sem_formulario = |n| recusa(ErroMarcar::RemocaoSemFormularioObservado, n);
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = match obter_tela(rede, &cookie, link) {
        Ok(tela) => tela,
        Err(FalhaDaTela::Transporte(erro)) => return Err(erro.into()),
        Err(
            FalhaDaTela::Redirecionada
            | FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada),
        ) => return Err(sem_formulario(rede.feitas())),
    };
    let Some(acao) = action_de_remocao(&html, &rotulo.id) else {
        return Err(sem_formulario(rede.feitas()));
    };
    let Some(codificacao) = codificacao_da_tela(&mime) else {
        return Err(sem_formulario(rede.feitas()));
    };
    let Some(instalacao) = Instalacao::interpretar(sessao.origem) else {
        return Err(sem_formulario(rede.feitas()));
    };
    let tela = TelaServida {
        url: link,
        html: &html,
        codificacao,
    };
    derivar_submissao_com_action(
        &REMOVER,
        &instalacao,
        &tela,
        unidade,
        &[("hdnInfraItemId", rotulo.id.as_str())],
        Some(&acao),
    )
    .map_err(Into::into)
}

/// Confirmação disponível apenas após a guarda de origem do comando.
/// Toda pré-condição é verificada antes do POST, que nunca é repetido.
pub(crate) fn confirmar_retirada<V: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    rotulo: &Rotulo,
    envio: &mut V,
) -> anyhow::Result<()> {
    let mut rede = Contador::novo(envio);
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos: PedidoDeAndamentos::Omitidos,
        propriedades: false,
        marcadores: true,
        unidade_exigida: true,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Estrita,
    };
    let (aberto, acoes) = abrir_para_escrever(sessao, &pedido, indice, &mut rede)?;
    let Some(unidade) = aberto.unidade.as_deref() else {
        return Err(recusa(
            ErroMarcar::UnidadeDaSessaoNaoPublicada,
            rede.feitas(),
        ));
    };
    let Marcadores::Lidos(ref lidos) = aberto.marcadores else {
        return Err(recusa(ErroMarcar::MarcadoresNaoLidos, rede.feitas()));
    };
    let decisao =
        decidir(lidos, rotulo, Operacao::Retirar).map_err(|e| recusa(e, rede.feitas()))?;
    if decisao == Decisao::NadaAFazer {
        saida_sem_alteracao(
            cli,
            nup,
            &aberto.arvore.processo.id,
            unidade,
            rotulo,
            rede.feitas(),
        );
        return Ok(());
    }
    let Some(link) = acoes.marcadores.ok().flatten() else {
        return Err(recusa(
            ErroMarcar::RemocaoSemFormularioObservado,
            rede.feitas(),
        ));
    };
    let submissao = preparar_submissao(sessao, &mut rede, link.as_str(), unidade, rotulo)?;
    let mut tentativa = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |submissao, cookie| {
        tentativa = true;
        rede.post(submissao, cookie)
    });
    if !tentativa {
        return Err(match resposta {
            Err(erro) => erro.into(),
            _ => recusa(ErroMarcar::RemocaoSemFormularioObservado, rede.feitas()),
        });
    }
    let aceito = matches!(
        resposta,
        Ok(RespostaDoEnvio::Redirecionamento(ref redir))
            if aceite_da_remocao(redir, &aberto.arvore.processo.id)
    );
    // Releitura independente, feita UMA vez mesmo em resposta HTTP incerta.
    let relida = obter_processo(sessao, &pedido, indice, &mut rede);
    let ausente = match relida {
        Ok(ref novo)
            if novo.arvore.processo.id == aberto.arvore.processo.id
                && novo.unidade.as_deref() == Some(unidade) =>
        {
            match &novo.marcadores {
                Marcadores::Lidos(lidos) if lidos.itens.len() == lidos.total_declarado => {
                    Some(!lidos.itens.iter().any(|m| m.id == rotulo.id))
                }
                Marcadores::Lidos(_)
                | Marcadores::Omitidos
                | Marcadores::NaoPublicadosNaBarra
                | Marcadores::TabelaNaoPublicada => None,
            }
        }
        _ => None,
    };
    if !aceito || ausente != Some(true) {
        return Err(FalhaDaMarcacao::RetiradaNaoConfirmada {
            requisicoes: rede.feitas(),
            ausente_relido: ausente,
        }
        .into());
    }
    saida_confirmada(
        cli,
        nup,
        &aberto.arvore.processo.id,
        unidade,
        rotulo,
        rede.feitas(),
    );
    Ok(())
}
