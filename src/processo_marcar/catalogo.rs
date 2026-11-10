//! BDD-255-01: atribuição ausente exige catálogo publicado no formulário da unidade.
//! Prévia não modifica processo e confirmação só será permitida após aceite causal.
use super::{Decisao, ErroMarcar, Operacao, Rotulo, decidir};
use crate::arvore_processo::endereco::conferir_endereco_relativo;
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
use crate::processo_concluir::escrita::{Contador, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, SessaoDoProcesso,
    abrir_para_escrever, obter_processo,
};
use crate::transporte::decodificar_resposta;
use crate::visao_do_controle::EnvioDoControle as _;
use crate::{Cli, saida_erro};
use serde_json::json;
use std::process::ExitCode;

const ADICIONAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "andamento_marcador_cadastrar",
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

fn falha(cli: &Cli, erro: ErroMarcar, reqs: u8) -> ExitCode {
    crate::processo_marcar_cli::falha(cli, erro, reqs)
}

/// Extrai link do botão efetivamente servido, sem executar JavaScript.
/// O link retornado só é usado depois da validação de ação, processo e assinatura.
fn link_de_adicionar(html: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let mut links = Vec::new();
    for tag in dom
        .query_selector("button")?
        .filter_map(|h| h.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        if valor_do_atributo(tag, "id").ok()?.as_deref() != Some("btnAdicionar") {
            continue;
        }
        let onclick = valor_do_atributo(tag, "onclick").ok()??;
        let relativo = onclick
            .strip_prefix("location.href='")?
            .strip_suffix('\'')?;
        links.push(decodificar_atributo(relativo)?);
    }
    match links.as_slice() {
        [link] => Some(link.clone()),
        _ => None,
    }
}

fn destino_cadastro<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    instalacao: &Instalacao,
    id: &str,
) -> Option<String> {
    let resposta = rede.get_sem_seguir(link, cookie).ok()?;
    let destino = if resposta.resposta.status_code == 302 {
        let redir = resposta.localizacao?;
        let prefixo = format!("{}/", instalacao.url_base());
        redir.strip_prefix(&prefixo).unwrap_or(&redir).to_owned()
    } else {
        let html = decodificar_resposta(resposta.resposta).ok()?;
        link_de_adicionar(&html)?
    };
    conferir_endereco_relativo(instalacao, &destino, ADICIONAR.acao, id).ok()
}

/// A identidade do catálogo vem de option[value=id] no select da unidade,
/// não do marcador aplicado ao processo nem de uma lista sem origem.
fn rotulo_oferecido(html: &str, rotulo: &Rotulo) -> bool {
    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return false;
    };
    let parser = dom.parser();
    let mut selects = dom
        .query_selector("select")
        .into_iter()
        .flatten()
        .filter_map(|h| h.get(parser).and_then(tl::Node::as_tag))
        .filter(|tag| {
            valor_do_atributo(tag, "id").ok().flatten().as_deref() == Some("selMarcador")
        });
    let Some(select) = selects.next() else {
        return false;
    };
    if selects.next().is_some() {
        return false;
    }
    let mut encontrados = 0;
    for tag in select
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|tag| tag.name().as_utf8_str() == "option")
    {
        let Ok(Some(id)) = valor_do_atributo(tag, "value") else {
            return false;
        };
        if id == rotulo.id {
            encontrados += 1;
            let nome = tag.inner_text(parser).to_string();
            if decodificar_atributo(&nome).as_deref() != Some(rotulo.nome.as_str()) {
                return false;
            }
        }
    }
    encontrados == 1
}

fn submissao_cadastro<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    link: &str,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
) -> Option<crate::formulario_web::SubmissaoDerivada> {
    let Ok(cookie) = ler_cookie(sessao) else {
        return None;
    };
    let instalacao = Instalacao::interpretar(sessao.origem)?;
    let url = destino_cadastro(rede, &cookie, link, &instalacao, id)?;
    let Ok((html, mime)) = obter_tela(rede, &cookie, &url) else {
        return None;
    };
    let codificacao = codificacao_da_tela(&mime)?;
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao,
    };
    if !rotulo_oferecido(&html, rotulo) {
        return None;
    }
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
    .ok()
}

fn saida_previa(
    cli: &Cli,
    nup: &NumeroProcesso,
    destino: (&str, &str),
    rotulo: &Rotulo,
    texto: Option<&str>,
    nada_a_fazer: bool,
    requisicoes: u8,
) -> ExitCode {
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
    ExitCode::SUCCESS
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
) -> ExitCode {
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
    let (aberto, acoes) = match abrir_para_escrever(sessao, &pedido, indice, &mut rede) {
        Ok(resposta) => resposta,
        Err(erro) => {
            return saida_erro::erro_fonte_http(cli.json, erro.mensagem(), erro.status_http());
        }
    };
    let Some(unidade) = aberto.unidade.as_deref() else {
        return falha(cli, ErroMarcar::UnidadeDaSessaoNaoPublicada, rede.feitas());
    };
    let Marcadores::Lidos(ref lidos) = aberto.marcadores else {
        return falha(cli, ErroMarcar::MarcadoresNaoLidos, rede.feitas());
    };
    let nada = match decidir(lidos, rotulo, Operacao::Atribuir) {
        Ok(Decisao::NadaAFazer) => true,
        Err(ErroMarcar::CatalogoDaUnidadeNaoObservado) => false,
        Ok(Decisao::PreviaDeRetirada) => unreachable!(),
        Err(e) => return falha(cli, e, rede.feitas()),
    };
    if nada {
        return saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            true,
            rede.feitas(),
        );
    }
    let Some(link) = acoes.marcadores.ok().flatten() else {
        return falha(
            cli,
            ErroMarcar::CatalogoDaUnidadeNaoObservado,
            rede.feitas(),
        );
    };
    let Some(submissao) = submissao_cadastro(
        sessao,
        &mut rede,
        link.as_str(),
        &aberto.arvore.processo.id,
        unidade,
        rotulo,
        texto,
    ) else {
        return falha(
            cli,
            ErroMarcar::CatalogoDaUnidadeNaoObservado,
            rede.feitas(),
        );
    };
    if !confirmar {
        return saida_previa(
            cli,
            nup,
            (&aberto.arvore.processo.id, unidade),
            rotulo,
            texto,
            false,
            rede.feitas(),
        );
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
) -> ExitCode {
    let (sessao, indice, pedido) = contexto;
    let (id, unidade) = destino;
    let (rotulo, texto) = marcador;
    let mut tentativa = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |submissao, cookie| {
        tentativa = true;
        rede.post(submissao, cookie)
    });
    if !tentativa {
        return falha(
            cli,
            ErroMarcar::CatalogoDaUnidadeNaoObservado,
            rede.feitas(),
        );
    }
    let aceito = matches!(
        resposta,
        Ok(RespostaDoEnvio::Redirecionamento(ref redir))
            if aceite_da_atribuicao(redir, id)
    );
    // Segunda leitura independente: não confiar no próprio POST nem repetir.
    let presente = releitura_comprova(sessao, indice, pedido, rede, (id, unidade), (rotulo, texto));
    if !aceito || presente != Some(true) {
        return efeito_incerto(cli, rede.feitas(), presente);
    }
    saida_confirmada(cli, nup, id, unidade, rotulo, texto, rede.feitas())
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
    acao == Some("andamento_marcador_gerenciar")
        && origem == Some("andamento_marcador_cadastrar")
        && procedimento == Some(id)
}

fn efeito_incerto(cli: &Cli, requisicoes: u8, presente_relido: Option<bool>) -> ExitCode {
    let mensagem = "o efeito desta tentativa não foi comprovado; não repita a confirmação; consulte o processo";
    if cli.json {
        println!(
            "{}",
            json!({
                "codigo": "efeito_incerto",
                "erro": mensagem,
                "retentavel": false,
                "requisicoes": requisicoes,
                "presente_relido": presente_relido,
            })
        );
    } else {
        eprintln!("IES: {mensagem}. Requisições: {requisicoes}.");
    }
    ExitCode::from(3)
}

fn saida_confirmada(
    cli: &Cli,
    nup: &NumeroProcesso,
    id: &str,
    unidade: &str,
    rotulo: &Rotulo,
    texto: Option<&str>,
    requisicoes: u8,
) -> ExitCode {
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
    ExitCode::SUCCESS
}

fn releitura_comprova<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
    pedido: &PedidoDoProcesso<'_>,
    rede: &mut Contador<'_, V>,
    processo: (&str, &str),
    marcador: (&Rotulo, Option<&str>),
) -> Option<bool> {
    let (id, unidade) = processo;
    let (rotulo, texto) = marcador;
    let releitura = obter_processo(sessao, pedido, indice, rede);
    match releitura {
        Ok(ref novo)
            if novo.arvore.processo.id == id && novo.unidade.as_deref() == Some(unidade) =>
        {
            match &novo.marcadores {
                Marcadores::Lidos(lidos) if lidos.itens.len() == lidos.total_declarado => {
                    Some(lidos.itens.iter().any(|m| {
                        m.id == rotulo.id
                            && m.nome == rotulo.nome
                            && m.texto.as_deref().unwrap_or("") == texto.unwrap_or("")
                    }))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
