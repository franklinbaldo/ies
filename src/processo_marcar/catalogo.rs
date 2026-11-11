//! BDD-255-01: atribuição ausente exige catálogo publicado no formulário da unidade.
//! Prévia não modifica processo e confirmação só será permitida após aceite causal.
use super::{
    Decisao, ErroMarcar, FalhaDaMarcacao, Operacao, Rotulo, decidir, recusa,
    releitura_de_marcadores,
};
use crate::Cli;
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, Redirecionamento302, RespostaDoEnvio, ResultadoDoEnvio, TelaServida,
    consulta_do_controlador, decodificar_atributo, derivar_submissao, enviar_submissao_via,
};
use crate::html_atributos::valor_do_atributo;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever,
};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::EnvioDoControle as _;
use serde_json::json;

const ADICIONAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: AcaoDoSei::AndamentoMarcadorCadastrar,
    formulario: "frmAndamentoMarcadorCadastro",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmSalvar"),
    campos_ocultos_exigidos: &["hdnInfraTipoPagina", "hdnIdMarcador", "hdnIdProtocolo"],
    campos_ocultos_reescritos: &["hdnIdMarcador"],
    // O ddslick substitui o select no envio; não se inventa selMarcador no POST.
    campos_usuario: &["txaTexto"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::MarcadoresNoProcesso,
    },
};

/// Extrai link do botão efetivamente servido, sem executar JavaScript.
/// O link retornado só é usado depois da validação de ação, processo e assinatura.
fn link_de_adicionar(html: &str) -> Result<String, ErroMarcar> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroMarcar::PaginaDoCadastroIlegivel)?;
    let botoes = dom
        .query_selector("button")
        .ok_or(ErroMarcar::CadastroDoMarcadorForaDaForma)?;
    let mut links = Vec::new();
    for tag in botoes.filter_map(|h| h.get(dom.parser()).and_then(tl::Node::as_tag)) {
        let id =
            valor_do_atributo(tag, "id").map_err(|_| ErroMarcar::CadastroDoMarcadorForaDaForma)?;
        if id.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick")
            .map_err(|_| ErroMarcar::CadastroDoMarcadorForaDaForma)?
            .ok_or(ErroMarcar::CadastroDoMarcadorForaDaForma)?;
        let relativo = onclick
            .strip_prefix("location.href='")
            .and_then(|resto| resto.strip_suffix('\''))
            .ok_or(ErroMarcar::CadastroDoMarcadorForaDaForma)?;
        links
            .push(decodificar_atributo(relativo).ok_or(ErroMarcar::CadastroDoMarcadorForaDaForma)?);
    }
    match links.as_slice() {
        [] => Err(ErroMarcar::BotaoAdicionarAusente),
        [link] => Ok(link.clone()),
        [_, ..] => Err(ErroMarcar::BotaoAdicionarRepetido),
    }
}

fn destino_cadastro<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    instalacao: &Instalacao,
    id: &str,
) -> anyhow::Result<String> {
    let resposta = rede.get_sem_seguir(link, cookie)?;
    let destino = if resposta.resposta.status_code == 302 {
        let redir = resposta.localizacao.ok_or_else(|| {
            recusa(
                ErroMarcar::RedirecionamentoDoCadastroSemDestino,
                rede.feitas(),
            )
        })?;
        let prefixo = format!("{}/", instalacao.url_base());
        redir.strip_prefix(&prefixo).unwrap_or(&redir).to_owned()
    } else {
        let html = decodificar_resposta(resposta.resposta)?;
        link_de_adicionar(&html).map_err(|causa| recusa(causa, rede.feitas()))?
    };
    conferir_endereco_relativo(instalacao, &destino, ADICIONAR.acao, id).map_err(|recusado| {
        let causa = match recusado {
            EnderecoRecusado::NaoReconhecido => ErroMarcar::EnderecoDoCadastroNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => ErroMarcar::EnderecoDoCadastroDeOutroProcesso,
        };
        recusa(causa, rede.feitas())
    })
}

/// A identidade do catálogo vem de option[value=id] no select da unidade,
/// não do marcador aplicado ao processo nem de uma lista sem origem.
fn rotulo_oferecido(html: &str, rotulo: &Rotulo) -> Result<(), ErroMarcar> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroMarcar::PaginaDoCadastroIlegivel)?;
    let parser = dom.parser();
    let mut selects = Vec::new();
    for tag in dom
        .query_selector("select")
        .ok_or(ErroMarcar::CadastroDoMarcadorForaDaForma)?
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
    {
        let id =
            valor_do_atributo(tag, "id").map_err(|_| ErroMarcar::CadastroDoMarcadorForaDaForma)?;
        if id.as_deref() == Some("selMarcador") {
            selects.push(tag);
        }
    }
    let [select] = selects.as_slice() else {
        return Err(ErroMarcar::SelectDeMarcadoresAusenteOuRepetido);
    };
    let mut encontrados = 0;
    for tag in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str() == "option")
    {
        let id = valor_do_atributo(tag, "value")
            .map_err(|_| ErroMarcar::CadastroDoMarcadorForaDaForma)?
            .ok_or(ErroMarcar::OpcaoDoMarcadorSemValor)?;
        if id == rotulo.id {
            encontrados += 1;
            let nome = tag.inner_text(parser).to_string();
            if decodificar_atributo(&nome).as_deref() != Some(rotulo.nome.as_str()) {
                return Err(ErroMarcar::NomeDoMarcadorDiferenteDoCatalogo);
            }
        }
    }
    match encontrados {
        1 => Ok(()),
        0 => Err(ErroMarcar::OpcaoDoMarcadorNaoOferecida),
        _ => Err(ErroMarcar::OpcaoDoMarcadorRepetida),
    }
}

fn submissao_cadastro<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    link: &str,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
) -> anyhow::Result<crate::formulario_web::SubmissaoDerivada> {
    let cookie = ler_cookie(sessao)?;
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or_else(|| recusa(ErroMarcar::OrigemNaoInterpretada, rede.feitas()))?;
    let url = destino_cadastro(rede, &cookie, link, &instalacao, id)?;
    let (html, mime) = match obter_tela(rede, &cookie, &url) {
        Ok(tela) => tela,
        Err(FalhaDaTela::Transporte(erro)) => return Err(erro.into()),
        Err(FalhaDaTela::Redirecionada | FalhaDaTela::SessaoEncerrada(_)) => {
            return Err(recusa(
                ErroMarcar::FormularioDeCadastroNaoLido,
                rede.feitas(),
            ));
        }
    };
    let codificacao = codificacao_da_tela(&mime)
        .ok_or_else(|| recusa(ErroMarcar::CharsetDoFormularioNaoSuportado, rede.feitas()))?;
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao,
    };
    rotulo_oferecido(&html, rotulo).map_err(|causa| recusa(causa, rede.feitas()))?;
    derivar_submissao(
        &ADICIONAR,
        &instalacao,
        &tela,
        unidade,
        &[
            ("hdnIdMarcador", &rotulo.id),
            ("txaTexto", texto.unwrap_or("")),
        ],
    )
    .map_err(Into::into)
}

fn saida_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    destino: (&str, &str),
    rotulo: &Rotulo,
    texto: Option<&str>,
    nada_a_fazer: bool,
    requisicoes: u8,
) {
    let (id, unidade) = destino;
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(), "id_procedimento": id, "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "texto": texto, "acao": "atribuir", "origem": "web",
                "confirmado": false, "alterado": false,
                "nada_a_fazer": nada_a_fazer,
                "retirada_possivel": false, "requisicoes": requisicoes,
            })
        );
    } else if nada_a_fazer {
        println!(
            "Processo {}: nada a fazer com {} (atribuir). Requisições: {}.",
            nup.as_str(),
            rotulo.id,
            requisicoes
        );
    } else {
        println!(
            "Prévia: atribuir marcador {}#{} ao processo {} (unidade {}, texto: {:?}). Requisições: {}. Esta prévia não alterou o processo. Para tentar a atribuição, use --confirmar em treinamento; o catálogo e o formulário precisam estar conferidos.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            texto.unwrap_or(""),
            requisicoes
        );
    }
}

/// Atribuição ausente só produz prévia se a opção exata estiver na tela
/// independente de cadastro da unidade, incluindo validação do form servido.
pub(crate) fn executar_atribuicao<V: EnvioDoProcesso>(
    cli: &Cli,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    nup: &NumeroProcesso,
    (rotulo, texto, confirmar): (&Rotulo, Option<&str>, bool),
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
    let nada = match decidir(lidos, rotulo, Operacao::Atribuir) {
        Ok(Decisao::NadaAFazer) => true,
        Ok(Decisao::PreviaDeAtribuicao) => false,
        Ok(Decisao::PreviaDeRetirada) => unreachable!(),
        Err(e) => return Err(recusa(e, rede.feitas())),
    };
    if nada {
        saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            true,
            rede.feitas(),
        );
        return Ok(());
    }
    let link = acoes
        .marcadores?
        .ok_or_else(|| recusa(ErroMarcar::LinkDeMarcadoresNaoPublicado, rede.feitas()))?;
    let submissao = submissao_cadastro(
        sessao,
        &mut rede,
        link.as_str(),
        &aberto.arvore.processo.id,
        unidade,
        rotulo,
        texto,
    )?;
    if !confirmar {
        saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            false,
            rede.feitas(),
        );
        return Ok(());
    }
    finalizar_atribuicao(
        cli,
        (sessao, indice, &pedido),
        nup,
        (&aberto.arvore.processo.id, unidade),
        (rotulo, texto),
        &mut rede,
        submissao,
    )
}

/// Após uma única tentativa, a resposta causal e a releitura são provas separadas.
fn finalizar_atribuicao<V: EnvioDoProcesso>(
    cli: &Cli,
    contexto: (&SessaoDoProcesso<'_>, &IndiceLocal, &PedidoDoProcesso<'_>),
    nup: &NumeroProcesso,
    destino: (&str, &str),
    marcador: (&Rotulo, Option<&str>),
    rede: &mut Contador<'_, V>,
    submissao: crate::formulario_web::SubmissaoDerivada,
) -> anyhow::Result<()> {
    let (sessao, indice, pedido) = contexto;
    let (id, unidade) = destino;
    let (rotulo, texto) = marcador;
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
            if aceite_da_atribuicao(redir, id)
    );
    // Segunda leitura independente: não confiar no próprio POST nem repetir.
    let presente =
        releitura_de_marcadores(sessao, indice, pedido, rede, (id, unidade)).map(|lidos| {
            lidos.itens.iter().any(|m| {
                m.id == rotulo.id
                    && m.nome == rotulo.nome
                    && m.texto.as_deref().unwrap_or("") == texto.unwrap_or("")
            })
        });
    if !aceito || !matches!(presente, Ok(true)) {
        return Err(FalhaDaMarcacao::AtribuicaoNaoConfirmada {
            requisicoes: rede.feitas(),
            presente_relido: presente,
        }
        .into());
    }
    saida_confirmada(cli, nup, id, unidade, rotulo, texto, rede.feitas());
    Ok(())
}

fn aceite_da_atribuicao(redir: &Redirecionamento302, id: &str) -> bool {
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
        && origem.and_then(AcaoDoSei::ler) == Some(AcaoDoSei::AndamentoMarcadorCadastrar)
        && procedimento == Some(id)
}

fn saida_confirmada(
    cli: &Cli,
    nup: &NumeroProcesso,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
    requisicoes: u8,
) {
    if cli.json {
        println!(
            "{}",
            json!({
                "nup": nup.as_str(), "id_procedimento": id, "unidade": unidade,
                "marcador": format!("{}#{}", rotulo.id, rotulo.nome),
                "texto": texto, "acao": "atribuir", "origem": "web",
                "confirmado": true, "alterado": true,
                "nada_a_fazer": false, "requisicoes": requisicoes,
            })
        );
    } else {
        println!(
            "Atribuição confirmada: marcador {}#{} no processo {} (unidade {}). Requisições: {}.",
            rotulo.id,
            rotulo.nome,
            nup.as_str(),
            unidade,
            requisicoes
        );
    }
}
