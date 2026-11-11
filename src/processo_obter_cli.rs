//! `ies processo NUP` (contracts/cli/processo-obter-arvore.md
//! e processo-obter-andamentos.md): lê os arquivos privados da sessão e o
//! índice local, abre o processo e publica identidade, anexados, documentos
//! e pastas da árvore, andamentos, propriedades
//! (processo-obter-propriedades.md) e marcadores
//! (processo-obter-marcadores.md). Nunca URL, hash ou HTML.
use crate::arvore_processo::{PastaNaArvore, VerificacaoDaPasta};
use crate::historico_processo::AndamentosLidos;
use crate::indice_local::IndiceLocal;
use crate::marcadores_processo::{MarcadoresLidos, rotulo_do_marcador};
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    Andamentos, ErroProcessoObter, Marcadores, PedidoDeAndamentos, PedidoDoProcesso,
    ProcessoAberto, Propriedades, SessaoDoProcesso,
};
use crate::propriedades_processo::PropriedadesLidas;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, configuracao, relogin_cli, transporte};
use serde_json::{Value, json};
use std::fmt::Write as _;

/// As propriedades lidas; ids de assunto e de contato não são publicados.
fn json_das_propriedades(lidas: &PropriedadesLidas) -> Value {
    let assuntos: Vec<Value> = lidas
        .assuntos
        .iter()
        .map(|a| json!({"codigo": a.codigo, "descricao": a.descricao}))
        .collect();
    let mut objeto = serde_json::Map::new();
    objeto.insert("tipo".to_owned(), json!(lidas.tipo));
    objeto.insert("especificacao".to_owned(), json!(lidas.especificacao));
    objeto.insert("interessados".to_owned(), json!(lidas.interessados));
    objeto.insert("assuntos".to_owned(), Value::Array(assuntos));
    objeto.insert("observacoes".to_owned(), json!(lidas.observacoes));
    objeto.insert(
        "nivel_de_acesso".to_owned(),
        json!(lidas.nivel_de_acesso.como_texto()),
    );
    // Sem hipótese legal, a chave sai (não há `null`).
    if let Some(hipotese) = &lidas.hipotese_legal {
        objeto.insert("hipotese_legal".to_owned(), json!(hipotese));
    }
    objeto.insert("data_de_geracao".to_owned(), json!(lidas.data_de_geracao));
    objeto.insert("fonte".to_owned(), json!(lidas.fonte.acao().como_na_url()));
    Value::Object(objeto)
}

/// Os marcadores lidos (sempre lista), o total da legenda e a faixa lida.
fn inserir_marcadores(objeto: &mut serde_json::Map<String, Value>, lidos: &MarcadoresLidos) {
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|m| {
            let mut marcador = serde_json::Map::new();
            marcador.insert("id".to_owned(), json!(m.id));
            marcador.insert(
                "rotulo".to_owned(),
                json!(rotulo_do_marcador(&m.id, &m.nome)),
            );
            marcador.insert("nome".to_owned(), json!(m.nome));
            marcador.insert("cor".to_owned(), json!(m.cor));
            if let Some(texto) = &m.texto {
                marcador.insert("texto".to_owned(), json!(texto));
            }
            if let Some(usuario) = &m.usuario {
                marcador.insert("usuario".to_owned(), json!(usuario));
            }
            marcador.insert("data_hora".to_owned(), json!(m.data_hora));
            Value::Object(marcador)
        })
        .collect();
    // Zero marcadores: a faixa lida não existe e a chave é omitida.
    let faixa = (!itens.is_empty()).then(|| json!({"inicio": 1, "fim": itens.len()}));
    objeto.insert("marcadores".to_owned(), Value::Array(itens));
    objeto.insert(
        "marcadores_total_declarado".to_owned(),
        json!(lidos.total_declarado),
    );
    if let Some(faixa) = faixa {
        objeto.insert("marcadores_faixa_observada".to_owned(), faixa);
    }
}

/// Linhas do texto humano dos marcadores: uma por marcador.
fn humano_dos_marcadores(marcadores: &Marcadores, texto: &mut String) {
    let Marcadores::Lidos(lidos) = marcadores else {
        if let Some(motivo) = marcadores.motivo() {
            let _ = write!(texto, "\nMarcadores não lidos ({motivo}).");
        }
        return;
    };
    let _ = write!(
        texto,
        "\nMarcadores: {} de {} declarados{}",
        lidos.itens.len(),
        lidos.total_declarado,
        if lidos.itens.is_empty() { "." } else { ":" }
    );
    for m in &lidos.itens {
        let _ = write!(
            texto,
            "\n- {} [{}]",
            rotulo_do_marcador(&m.id, &m.nome),
            m.cor
        );
        if let Some(t) = &m.texto {
            let _ = write!(texto, ": {t}");
        }
    }
}

/// Linhas do texto humano das propriedades.
fn humano_das_propriedades(propriedades: &Propriedades, texto: &mut String) {
    let Propriedades::Lidas(lidas) = propriedades else {
        if let Some(motivo) = propriedades.motivo() {
            let _ = write!(texto, "\nPropriedades não lidas ({motivo}).");
        }
        return;
    };
    let _ = write!(
        texto,
        "\nPropriedades ({}): tipo {}, nível {}, gerado em {}.",
        lidas.fonte.acao().como_na_url(),
        lidas.tipo,
        lidas.nivel_de_acesso.como_texto(),
        lidas.data_de_geracao,
    );
    if let Some(hipotese) = &lidas.hipotese_legal {
        let _ = write!(texto, "\nHipótese legal: {hipotese}");
    }
    let _ = write!(texto, "\nEspecificação: {}", lidas.especificacao);
    for a in &lidas.assuntos {
        let _ = write!(texto, "\n- assunto {} - {}", a.codigo, a.descricao);
    }
    for interessado in &lidas.interessados {
        let _ = write!(texto, "\n- interessado {interessado}");
    }
    if !lidas.observacoes.is_empty() {
        let _ = write!(texto, "\nObservações: {}", lidas.observacoes);
    }
}

fn json_dos_andamentos(lidos: &AndamentosLidos) -> Value {
    // Zero andamentos: a faixa lida não existe e a chave é omitida.
    let faixa = (!lidos.itens.is_empty()).then(|| json!({"inicio": 1, "fim": lidos.itens.len()}));
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|a| {
            let mut andamento = serde_json::Map::new();
            andamento.insert("ocorrido_em".to_owned(), json!(a.ocorrido_em));
            if let Some(unidade) = &a.unidade {
                andamento.insert("unidade".to_owned(), json!(unidade));
            }
            if let Some(usuario) = &a.usuario {
                andamento.insert("usuario".to_owned(), json!(usuario));
            }
            andamento.insert("descricao".to_owned(), json!(a.descricao));
            Value::Object(andamento)
        })
        .collect();
    let mut objeto = json!({
        "tipo": lidos.tipo.como_texto(),
        "total_declarado": lidos.total_declarado,
        "itens": itens,
    });
    if let (Some(campos), Some(faixa)) = (objeto.as_object_mut(), faixa) {
        campos.insert("faixa_observada".to_owned(), faixa);
    }
    objeto
}

/// A verificação mais fraca entre as pastas conferidas (`por_contagem` <
/// `mista` < `por_identidade`); `None` sem pastas.
fn verificacao_das_pastas(pastas: &[PastaNaArvore]) -> Option<VerificacaoDaPasta> {
    use VerificacaoDaPasta::{Mista, PorContagem, PorIdentidade};
    let regras: Vec<VerificacaoDaPasta> = pastas.iter().filter_map(|p| p.verificacao).collect();
    [PorContagem, Mista, PorIdentidade]
        .into_iter()
        .find(|v| regras.contains(v))
}

/// Andamentos, marcadores e propriedades: o valor lido, ou só o motivo
/// quando o usuário os omitiu (`--sem-*`); `nao_lidos` em ordem alfabética
/// (depois de `aberto_em`). Leitura que falha não chega aqui.
fn inserir_leituras(
    objeto: &mut serde_json::Map<String, Value>,
    aberto: &ProcessoAberto,
    nao_lidos: &mut Vec<&str>,
) {
    match &aberto.andamentos {
        Andamentos::Lidos(lidos) => {
            objeto.insert("andamentos".to_owned(), json_dos_andamentos(lidos));
        }
        Andamentos::Omitidos => {}
    }
    if let Some(motivo) = aberto.andamentos.motivo() {
        nao_lidos.push("andamentos");
        objeto.insert("andamentos_nao_lidos_motivo".to_owned(), json!(motivo));
    }
    match &aberto.marcadores {
        Marcadores::Lidos(lidos) => inserir_marcadores(objeto, lidos),
        Marcadores::Omitidos => {}
    }
    if let Some(motivo) = aberto.marcadores.motivo() {
        nao_lidos.push("marcadores");
        objeto.insert("marcadores_nao_lidos_motivo".to_owned(), json!(motivo));
    }
    match &aberto.propriedades {
        Propriedades::Lidas(lidas) => {
            objeto.insert("propriedades".to_owned(), json_das_propriedades(lidas));
        }
        Propriedades::Omitidas => {}
    }
    if let Some(motivo) = aberto.propriedades.motivo() {
        nao_lidos.push("propriedades");
        objeto.insert("propriedades_nao_lidas_motivo".to_owned(), json!(motivo));
    }
}

/// O JSON publicado; `anexados` sai sempre; `documentos` some com `--sem-documentos`,
/// `andamentos` com `--sem-andamentos`, `propriedades` com `--sem-propriedades`
/// e `marcadores` com `--sem-marcadores` (link não publicado: chave ausente).
pub(crate) fn json_do_processo(
    nup: &NumeroProcesso,
    aberto: &ProcessoAberto,
    sem_documentos: bool,
) -> Value {
    let processo = &aberto.arvore.processo;
    let mut valor = json!({
        "numero": nup.as_str(),
        "id_procedimento": processo.id,
        "tipo": processo.tipo,
        "resolvido_por": aberto.resolvido_por.como_texto(),
        "requisicoes": aberto.requisicoes,
    });
    let mut nao_lidos: Vec<&str> = Vec::new();
    if let Some(objeto) = valor.as_object_mut() {
        // Sem unidade publicada pela árvore, a chave sai (não há `null`).
        if let Some(unidade) = &aberto.unidade {
            objeto.insert("unidade".to_owned(), json!(unidade));
        }
        saida_da_arvore::inserir_aberto_em(objeto, &aberto.arvore.aberto_em, &mut nao_lidos);
        inserir_leituras(objeto, aberto, &mut nao_lidos);
        let anexados = &aberto.arvore.anexados;
        if !anexados.is_empty() {
            nao_lidos.push("documentos_dos_anexados");
        }
        objeto.insert("nao_lidos".to_owned(), json!(nao_lidos));
        let anexados: Vec<Value> = anexados
            .iter()
            .map(|a| json!({"numero": a.nup, "id_procedimento": a.id, "tipo": a.tipo}))
            .collect();
        objeto.insert("anexados".to_owned(), Value::Array(anexados));
        let pai = aberto.arvore.anexado_a.as_ref();
        objeto.insert("anexado".to_owned(), json!(pai.is_some()));
        // Processo não anexado: a chave `anexado_a` sai (não há `null`).
        if let Some(p) = pai {
            objeto.insert(
                "anexado_a".to_owned(),
                json!({"numero": p.numero, "id_procedimento": p.id_procedimento}),
            );
        }
    }
    if !sem_documentos && let Some(objeto) = valor.as_object_mut() {
        saida_da_arvore::inserir_documentos(objeto, &aberto.arvore);
        let pastas: Vec<Value> = if aberto.arvore.documentos_incluidos {
            aberto
                .arvore
                .pastas
                .iter()
                .map(|p| {
                    let mut pasta = serde_json::Map::new();
                    pasta.insert("id".to_owned(), json!(p.id));
                    pasta.insert("rotulo".to_owned(), json!(p.rotulo));
                    pasta.insert("total".to_owned(), json!(p.total_declarado));
                    // Pasta sem regra de verificação: a chave sai (não há `null`).
                    if let Some(verificacao) = p.verificacao {
                        pasta.insert(
                            "verificacao".to_owned(),
                            json!(VerificacaoDaPasta::como_texto(verificacao)),
                        );
                    }
                    pasta.insert(
                        "conferidos_por_identidade".to_owned(),
                        json!(p.conferidos_por_identidade),
                    );
                    pasta.insert(
                        "conferidos_por_contagem".to_owned(),
                        json!(p.conferidos_por_contagem),
                    );
                    Value::Object(pasta)
                })
                .collect()
        } else {
            Vec::new()
        };
        objeto.insert("pastas".to_owned(), Value::Array(pastas));
        let verificacao = if aberto.arvore.documentos_incluidos {
            verificacao_das_pastas(&aberto.arvore.pastas)
        } else {
            None
        };
        // Sem pastas conferidas, a chave sai (não há `null`).
        if let Some(verificacao) = verificacao {
            objeto.insert(
                "verificacao_das_pastas".to_owned(),
                json!(verificacao.como_texto()),
            );
        }
        if verificacao.is_some_and(|v| v != VerificacaoDaPasta::PorIdentidade) {
            objeto.insert(
                "verificacao_das_pastas_motivo".to_owned(),
                json!("documentos_sem_identidade_na_barra"),
            );
        }
    }
    valor
}

fn humano(nup: &NumeroProcesso, aberto: &ProcessoAberto, sem_documentos: bool) -> String {
    let processo = &aberto.arvore.processo;
    let unidade = aberto
        .unidade
        .as_deref()
        .unwrap_or("não publicada pela árvore");
    let mut texto = format!(
        "IES: processo {} (id {}, {}), unidade {unidade}; resolvido por {} em {} requisições.",
        nup.as_str(),
        processo.id,
        processo.tipo,
        aberto.resolvido_por.como_texto(),
        aberto.requisicoes,
    );
    saida_da_arvore::humano_do_aberto_em(&aberto.arvore.aberto_em, &mut texto);
    if let Some(pai) = &aberto.arvore.anexado_a {
        let _ = write!(
            texto,
            "\nAnexado ao processo {} (id {}): ies processo {}",
            pai.numero, pai.id_procedimento, pai.numero
        );
    }
    let anexados = &aberto.arvore.anexados;
    if !anexados.is_empty() {
        let _ = write!(
            texto,
            "\n{} processo(s) anexado(s), sem os documentos deles; abra cada um:",
            anexados.len()
        );
        for a in anexados {
            let _ = write!(
                texto,
                "\n- {} (id {}, {}): ies processo {}",
                a.nup, a.id, a.tipo, a.nup
            );
        }
    }
    if !sem_documentos {
        let arvore = &aberto.arvore;
        if arvore.documentos_incluidos {
            let _ = write!(
                texto,
                "\n{} documento(s) na árvore, {} pasta(s){}:",
                arvore.documentos.len(),
                arvore.pastas.len(),
                verificacao_das_pastas(&arvore.pastas)
                    .map(|v| format!(" conferidas {}", v.como_texto()))
                    .unwrap_or_default()
            );
            saida_da_arvore::humano_dos_documentos(arvore, &mut texto);
            saida_da_arvore::humano_dos_movidos(&arvore.documentos_movidos, &mut texto);
        } else {
            saida_da_arvore::humano_dos_documentos(arvore, &mut texto);
        }
    }
    match &aberto.andamentos {
        Andamentos::Lidos(lidos) => {
            let _ = write!(
                texto,
                "\nAndamentos (visão {}): {} de {} declarados.",
                lidos.tipo.como_texto(),
                lidos.itens.len(),
                lidos.total_declarado
            );
            for a in &lidos.itens {
                let _ = write!(
                    texto,
                    "\n- {} {}: {}",
                    a.ocorrido_em,
                    a.unidade.as_deref().unwrap_or("?"),
                    a.descricao
                );
            }
        }
        Andamentos::Omitidos => {
            if let Some(motivo) = aberto.andamentos.motivo() {
                let _ = write!(texto, "\nAndamentos não lidos ({motivo}).");
            }
        }
    }
    humano_das_propriedades(&aberto.propriedades, &mut texto);
    humano_dos_marcadores(&aberto.marcadores, &mut texto);
    texto
}

/// NUP e opções já conferidos pelo clap (`--sem-andamentos` exclui
/// `--andamentos-completos`).
fn obter(cli: &Cli, comando: &Processo, nup: &NumeroProcesso) -> anyhow::Result<()> {
    let andamentos = if comando.sem_andamentos {
        PedidoDeAndamentos::Omitidos
    } else if comando.andamentos_completos {
        PedidoDeAndamentos::Completos
    } else {
        PedidoDeAndamentos::VisaoServida
    };
    let arquivos = configuracao::arquivos_de_acesso()?;
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos,
        propriedades: !comando.sem_propriedades,
        marcadores: !comando.sem_marcadores,
        unidade_exigida: false,
        leitura_das_pastas: crate::processo_obter::LeituraDasPastas::Informativa,
    };
    // Cada tentativa relê a sessão e o índice: depois do relogin, a nova.
    let (mut aberto, nota) = relogin_cli::ler(
        &arquivos,
        cli.unidade.as_deref(),
        ErroProcessoObter::sessao_encerrada,
        || {
            let referencias = transporte::sessao::ReferenciasDaSessao::ler(
                &arquivos.url_controle,
                &arquivos.origem,
                &arquivos.cookie,
            )
            .map_err(ErroProcessoObter::Transporte)?;
            let sessao = SessaoDoProcesso {
                url_do_controle: referencias.url_do_controle(),
                origem: referencias.origem(),
                cookie: referencias.cookie(),
            };
            let indice = IndiceLocal::da_configuracao(&arquivos);
            crate::processo_obter::consulta_parcial::obter_processo_informativo(
                &sessao,
                &pedido,
                &indice,
                &mut EnvioHttps,
            )
        },
    )
    .map_err(validacao_sem_credenciais)?;
    aberto.requisicoes = relogin_cli::somar(aberto.requisicoes, nota.as_ref());
    // A leitura de `aberto_em` que falhou é erro nos dois modos de saída.
    saida_da_arvore::exigir_aberto_em_lido(&aberto.arvore.aberto_em)?;
    if cli.json {
        let mut valor = json_do_processo(nup, &aberto, comando.sem_documentos);
        relogin_cli::anotar(&mut valor, nota.as_ref());
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else {
        relogin_cli::avisar(cli.json, nota.as_ref());
        println!("{}", humano(nup, &aberto, comando.sem_documentos));
    }
    Ok(())
}

/// O SEI exige validar o acesso com a senha e não há arquivo de credenciais
/// privado (`IES_CREDENCIAIS_FILE`); o IES ainda não faz essa validação.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct ValidacaoDeAcessoSemArquivoDeCredenciais;

/// A falha que sobe; a da validação de acesso leva junto a ausência do
/// arquivo de credenciais, quando é o caso.
fn validacao_sem_credenciais(
    falha: relogin_cli::FalhaDaLeitura<ErroProcessoObter>,
) -> anyhow::Error {
    let exige_validacao = matches!(
        &falha,
        relogin_cli::FalhaDaLeitura::Leitura {
            erro: ErroProcessoObter::Tela(
                crate::arvore_processo::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso
            ),
            ..
        }
    );
    let sem_credenciais =
        std::env::var_os("IES_CREDENCIAIS_FILE").is_none_or(|caminho| caminho.is_empty());
    let erro = relogin_cli::subir(falha);
    if exige_validacao && sem_credenciais {
        erro.context(ValidacaoDeAcessoSemArquivoDeCredenciais)
    } else {
        erro
    }
}

/// `Some` só para `ies processo NUP`.
pub(crate) fn tentar_obter(cli: &Cli) -> Option<anyhow::Result<()>> {
    let Domain::Processo(
        comando @ Processo {
            numero: Some(numero),
            acao: None,
            ..
        },
    ) = &cli.command
    else {
        return None;
    };
    Some(obter(cli, comando, numero))
}

#[path = "processo_obter_cli/saida_da_arvore.rs"]
mod saida_da_arvore;

#[cfg(test)]
#[path = "processo_obter_cli/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "processo_obter_cli/tests_marcadores.rs"]
mod tests_marcadores;
#[cfg(test)]
#[path = "processo_obter_cli/tests_movidos.rs"]
mod tests_movidos;
#[cfg(test)]
#[path = "processo_obter_cli/tests_propriedades.rs"]
mod tests_propriedades;
#[cfg(test)]
#[path = "processo_obter_cli/tests_situacao.rs"]
mod tests_situacao;
