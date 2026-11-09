//! Exclusao do prazo pela lista da unidade, nunca pelo cadastro da barra.
//! O unico POST e reservado ao anti-IES de teste; release recusa antes da rede.
use super::*;
use crate::formulario_web::derivar_submissao_com_action;
use crate::url_observada::UrlHttps;

const EXCLUIR_PRAZO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "controle_prazo_excluir",
    formulario: "frmControlePrazoLista",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraItemId"],
    campos_ocultos_reescritos: &["hdnInfraItemId"],
    campos_usuario: &[],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::ControleDePrazo,
    },
};

struct ExclusaoPreparada {
    link_lista: String,
    html_lista: String,
    mime: String,
    id_controle: String,
    id_procedimento: String,
    link_prazo: String,
}

/// Lê o href realmente publicado na pagina do Controle, sem fabricar token.
fn link_da_lista(html: &str, origem: &str) -> Option<String> {
    let dom = tl::parse(html, tl::ParserOptions::default()).ok()?;
    let instalacao = Instalacao::interpretar(origem)?;
    let mut encontrados = Vec::new();
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let bruto = crate::html_atributos::valor_do_atributo(ancora, "href")
            .ok()
            .flatten()?;
        let href = crate::formulario_web::decodificar_atributo(&bruto)?;
        if !href.split(['?', '&']).any(|parte| parte == "acao=controle_prazo_listar") {
            continue;
        }
        if href.starts_with('/') || href.contains("://") || href.contains("..") {
            return None;
        }
        let url = format!("{origem}/{href}");
        let alvo = UrlHttps::interpretar(&url)?;
        if !instalacao.autoriza_controlador(alvo.host, alvo.caminho) {
            return None;
        }
        let consulta = alvo.consulta?;
        let hashes: Vec<_> = consulta
            .split('&')
            .filter_map(|parte| parte.strip_prefix("infra_hash="))
            .collect();
        if hashes.len() != 1
            || hashes[0].len() != 64
            || !hashes[0].bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return None;
        }
        encontrados.push(url);
    }
    (encontrados.len() == 1).then(|| encontrados.remove(0))
}

/// Lê uma funcao de linha observada sem executar o JavaScript.
/// Ausencia de acao ao confirmar a exclusao significa ausencia da linha.
fn id_na_lista(html: &str, nup: &str) -> Result<Option<String>, ()> {
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ())?;
    crate::formulario_web::localizar_formulario(&dom, "frmControlePrazoLista").map_err(|_| ())?;
    let mut encontrado = None;
    for ancora in dom
        .query_selector("a")
        .into_iter()
        .flatten()
        .filter_map(|n| n.get(dom.parser()).and_then(tl::Node::as_tag))
    {
        let Some(acao) = crate::html_atributos::valor_do_atributo(ancora, "onclick")
            .map_err(|_| ())?
        else {
            continue;
        };
        let Some(resto) = acao.trim().strip_prefix("acaoExcluir('") else {
            continue;
        };
        let Some((id, numero)) = resto.split_once("','") else {
            return Err(());
        };
        let Some(numero) = numero.strip_suffix("');").or_else(|| numero.strip_suffix("')")) else {
            return Err(());
        };
        if numero != nup {
            continue;
        }
        if id.is_empty()
            || !id.bytes().all(|c| c.is_ascii_digit())
            || encontrado.replace(id.to_owned()).is_some()
        {
            return Err(());
        }
    }
    Ok(encontrado)
}

/// A URL literal do script passa pela mesma validacao da origem e do hash
/// usada pelo primitivo de formulario, nao por um executor de JavaScript.
fn action_de_exclusao(html: &str) -> Option<String> {
    let prefixo = "controlador.php?acao=controle_prazo_excluir&infra_hash=";
    let resto = html.split_once(prefixo)?.1;
    let hash = resto
        .bytes()
        .take_while(u8::is_ascii_hexdigit)
        .collect::<Vec<_>>();
    let hash = String::from_utf8(hash).ok()?;
    (hash.len() == 64).then(|| format!("{prefixo}{hash}"))
}

fn escrever_e_verificar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    dados: &ExclusaoPreparada,
    rede: &mut Contador<'_, V>,
    cookie: &str,
    json_mode: bool,
) -> Result<(), ExitCode> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or_else(|| recusar(json_mode, "tela_de_prazo_nao_observada", "origem invalida"))?;
    let codificacao = codificacao_da_tela(&dados.mime).ok_or_else(|| {
        recusar(json_mode, "tela_de_prazo_nao_observada", "charset da lista nao observado")
    })?;
    let url_declarada = action_de_exclusao(&dados.html_lista).ok_or_else(|| {
        recusar(json_mode, "tela_de_prazo_nao_observada", "a lista nao publicou a acao de exclusao")
    })?;
    let servida = TelaServida {
        url: &dados.link_lista,
        html: &dados.html_lista,
        codificacao,
    };
    let submissao = derivar_submissao_com_action(
        &EXCLUIR_PRAZO,
        &instalacao,
        &servida,
        "",
        &[("hdnInfraItemId", dados.id_controle.as_str())],
        Some(&url_declarada),
    )
    .map_err(|erro| {
        recusar(
            json_mode,
            erro.subcodigo(),
            "formulario de exclusao nao pode ser submetido",
        )
    })?;
    let mut enviou = false;
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |post, c| {
        enviou = true;
        rede.post(post, c)
    })
    .map_err(|erro| {
        if enviou {
            efeito_incerto(json_mode, Some(&erro))
        } else {
            recusar(json_mode, erro.subcodigo(), "envio de exclusao nao preparado")
        }
    })?;
    match resposta {
        RespostaDoEnvio::Redirecionamento(redir)
            if redir.localizacao().is_some_and(|local| {
                local.contains("acao=controle_prazo_listar")
                    && local.contains("acao_origem=controle_prazo_excluir")
            }) => {}
        _ => return Err(efeito_incerto(json_mode, None)),
    }
    let (lista_nova, _) = obter_tela(rede, cookie, &dados.link_lista)
        .map_err(|_| efeito_incerto(json_mode, None))?;
    if id_na_lista(&lista_nova, "").is_err()
        || id_na_lista(&lista_nova, &dados.id_procedimento).is_err()
    {
        return Err(efeito_incerto(json_mode, None));
    }
    // A lista nao deve mais oferecer o mesmo item. Conferir o id controle
    // original tambem evita declarar sucesso ao ocultar so a acao.
    if lista_nova.contains(&format!("acaoExcluir('{}',", dados.id_controle)) {
        return Err(efeito_incerto(json_mode, None));
    }
    // O GET do formulario de definicao e testemunha independente da lista.
    let (formulario, _) = obter_tela(rede, cookie, &dados.link_prazo)
        .map_err(|_| efeito_incerto(json_mode, None))?;
    let prazo = ler_tela(&formulario, &dados.id_procedimento)
        .map_err(|_| efeito_incerto(json_mode, None))?;
    if prazo.estado != EstadoDoPrazo::SemPrazo {
        return Err(efeito_incerto(json_mode, None));
    }
    Ok(())
}

/// Executa BDD-272-03 na CLI real, incluindo a previa sem qualquer POST.
pub(super) fn executar(
    cli: &Cli,
    nup: &NumeroProcesso,
    confirmar: bool,
    sessao: &SessaoDoProcesso<'_>,
    indice: &IndiceLocal,
) -> ExitCode {
    let mut envio = EnvioHttps;
    let mut rede = Contador::novo(&mut envio);
    let leitura = leitura(nup, cli.unidade.as_deref(), true);
    let (aberto, acoes) = match abrir_para_escrever(sessao, &leitura, indice, &mut rede) {
        Ok(aberto) => aberto,
        Err(erro) => {
            return saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            );
        }
    };
    let Ok(Some(link_prazo)) = acoes.prazo else {
        return recusar(
            cli.json,
            "prazo_nao_publicado_na_barra",
            "processo nao oferece leitura independente do prazo",
        );
    };
    let cookie = match ler_cookie(sessao) {
        Ok(cookie) => cookie,
        Err(erro) => {
            return saida_erro::erro_fonte_http(
                cli.json,
                erro.subcodigo(),
                erro.mensagem(),
                erro.status_http(),
            );
        }
    };
    let (controle, _) =
        match consultar_formulario(&mut rede, &cookie, sessao.url_do_controle, cli.json) {
            Ok(tela) => tela,
            Err(erro) => return erro,
        };
    let Some(link_lista) = link_da_lista(&controle, sessao.origem) else {
        return recusar(
            cli.json,
            "exclusao_de_prazo_ainda_nao_disponivel",
            "controle da unidade nao publicou lista de prazos",
        );
    };
    let (html_lista, mime) = match consultar_formulario(&mut rede, &cookie, &link_lista, cli.json)
    {
        Ok(tela) => tela,
        Err(erro) => return erro,
    };
    let id_controle = match id_na_lista(&html_lista, nup.as_str()) {
        Ok(Some(id)) => id,
        Ok(None) => {
            return recusar(
                cli.json,
                "exclusao_de_prazo_ainda_nao_disponivel",
                "o processo nao esta na lista de prazos da unidade",
            );
        }
        Err(()) => {
            return recusar(
                cli.json,
                "tela_de_prazo_nao_observada",
                "a lista de prazos nao identifica um unico controle",
            );
        }
    };
    let preparada = ExclusaoPreparada {
        link_lista,
        html_lista,
        mime,
        id_controle,
        id_procedimento: aberto.arvore.processo.id.clone(),
        link_prazo,
    };
    if confirmar
        && let Err(erro) = escrever_e_verificar(sessao, &preparada, &mut rede, &cookie, cli.json)
    {
        return erro;
    }
    let resultado = json!({
        "nup": nup.as_str(),
        "id_procedimento": aberto.arvore.processo.id,
        "unidade": aberto.unidade,
        "origem": "web",
        "atual": if confirmar { json!({"tipo": "sem_prazo"}) } else { Value::Null },
        "solicitado": {"excluir": true},
        "confirmado": confirmar,
        "requisicoes": rede.feitas(),
    });
    if cli.json {
        println!("{resultado}");
    } else if confirmar {
        println!("Prazo do processo {} excluido e verificado por leitura.", nup.as_str());
    } else {
        println!("Previa: excluir o controle de prazo de {}. Sem escrita.", nup.as_str());
    }
    ExitCode::SUCCESS
}
