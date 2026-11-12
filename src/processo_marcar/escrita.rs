//! BDD-255-02: uma única remoção, derivada do formulário efetivamente servido.
//! O GET de gerenciamento não autoriza POST sem handler, hidden e ação assinada.
use super::{
    Decisao, ErroMarcar, FalhaDaMarcacao, Operacao, Rotulo, decidir, recusa,
    releitura_de_marcadores,
};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, Redirecionamento302, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, derivar_submissao_com_action, enviar_submissao_via,
};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever,
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

/// Corpo servido de `function acaoRemover(id,desc)`: do fim da assinatura até a
/// próxima palavra `function` do script (ou o fim do script). Delimitado por
/// `function` e não por chaves, porque o SEI põe `{`/`}` dentro de strings do
/// corpo; a função seguinte, `acaoRemocaoMultipla`, atribui a mesma ação e
/// não pertence a este corpo. Se o corpo cortar errado, a recusa é fechada:
/// some a atribuição (Ausente) ou sobra outra (Ambigua).
fn corpo_de_acao_remover(html: &str) -> Result<&str, ErroMarcar> {
    let apos_assinatura = html
        .split_once("function acaoRemover(id,desc)")
        .ok_or(ErroMarcar::AcaoDeRemocaoAusente)?
        .1;
    Ok(apos_assinatura
        .split_once("function")
        .map_or(apos_assinatura, |(corpo, _)| corpo))
}

/// Extrai SOMENTE a URL literal assinada atribuída pelo handler da mesma tela.
/// Não executa JS, não troca host, não cria destino a partir de padrões.
/// Exige exatamente uma atribuição de `action` dentro do corpo de `acaoRemover`.
pub(super) fn action_de_remocao(html: &str, id: &str) -> Result<String, ErroMarcar> {
    let corpo = corpo_de_acao_remover(html)?;
    let inicio = "document.getElementById('frmGerenciarMarcador').action='";
    let url = corpo
        .split_once(inicio)
        .ok_or(ErroMarcar::AcaoDeRemocaoAusente)?
        .1
        .split_once('\'')
        .ok_or(ErroMarcar::AcaoDeRemocaoAusente)?
        .0;
    if !html.contains(&format!("acaoRemover('{id}',")) || url.is_empty() {
        return Err(ErroMarcar::AcaoDeRemocaoAusente);
    }
    if corpo.matches(inicio).count() != 1 {
        return Err(ErroMarcar::AcaoDeRemocaoAmbigua);
    }
    Ok(url.to_owned())
}

pub(super) fn aceite_da_remocao(redir: &Redirecionamento302, id: &str) -> bool {
    let Some(consulta) = redir
        .localizacao()
        .and_then(|url| consulta_do_controlador(url, redir.url_do_envio()))
    else {
        return false;
    };
    let Ok(consulta) = ConsultaDoControlador::ler(consulta) else {
        return false;
    };
    consulta.acao() == Ok(AcaoDoSei::AndamentoMarcadorGerenciar)
        && consulta.acao_de_origem() == Ok(AcaoDoSei::AndamentoMarcadorRemover)
        && consulta.unico("id_procedimento") == Ok(id)
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
    cookie: &str,
) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {
    let (html, mime) = match obter_tela(rede, cookie, link) {
        Ok(tela) => tela,
        Err(FalhaDaTela::Transporte(erro)) => return Err(erro.into()),
        Err(FalhaDaTela::Redirecionada | FalhaDaTela::SessaoEncerrada(_)) => {
            return Err(recusa(ErroMarcar::TelaDaRemocaoNaoLida, rede.feitas()));
        }
    };
    let acao =
        action_de_remocao(&html, &rotulo.id).map_err(|causa| recusa(causa, rede.feitas()))?;
    let codificacao = codificacao_da_tela(&mime)
        .ok_or_else(|| recusa(ErroMarcar::CharsetDoFormularioNaoSuportado, rede.feitas()))?;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or_else(|| recusa(ErroMarcar::OrigemNaoInterpretada, rede.feitas()))?;
    let tela = TelaServida {
        url: link,
        html: &html,
        codificacao,
    };
    derivar_submissao_com_action(
        &REMOVER,
        &instalacao,
        &tela,
        Some(unidade),
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
    let link = acoes
        .marcadores?
        .ok_or_else(|| recusa(ErroMarcar::LinkDeMarcadoresNaoPublicado, rede.feitas()))?;
    let submissao = preparar_submissao(
        sessao,
        &mut rede,
        link.as_str(),
        unidade,
        rotulo,
        &acoes.cookie,
    )?;
    let mut tentativa = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |submissao, cookie| {
        tentativa = true;
        rede.post(submissao, cookie)
    });
    if !tentativa {
        return Err(match resposta {
            Err(erro) => erro.into(),
            Ok(_) => recusa(ErroMarcar::SubmissaoDoMarcadorNaoEnviada, rede.feitas()),
        });
    }
    let aceito = matches!(
        resposta,
        Ok(RespostaDoEnvio::Redirecionamento(ref redir))
            if aceite_da_remocao(redir, &aberto.arvore.processo.id)
    );
    // Releitura independente, feita UMA vez mesmo em resposta HTTP incerta.
    let relida = releitura_de_marcadores(
        sessao,
        indice,
        &pedido,
        &mut rede,
        (&aberto.arvore.processo.id, unidade),
    )
    .map(|lidos| !lidos.itens.iter().any(|m| m.id == rotulo.id));
    if !aceito || !matches!(relida, Ok(true)) {
        return Err(FalhaDaMarcacao::RetiradaNaoConfirmada {
            requisicoes: rede.feitas(),
            ausente_relido: relida,
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
