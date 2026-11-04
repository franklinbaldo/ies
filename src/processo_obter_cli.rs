//! `ies processo NUP` sem `--resumo` (contracts/cli/processo-obter-arvore.md
//! e processo-obter-andamentos.md): lê os arquivos privados da sessão e o
//! índice local, abre o processo e publica identidade, anexados, documentos
//! e pastas da árvore, andamentos, propriedades
//! (processo-obter-propriedades.md) e marcadores
//! (processo-obter-marcadores.md). Nunca URL, hash ou HTML.
use crate::arvore_processo::{DocumentoMovido, PastaNaArvore, VerificacaoDaPasta};
use crate::historico_processo::AndamentosLidos;
use crate::indice_local::IndiceLocal;
use crate::marcadores_processo::MarcadoresLidos;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    Andamentos, Marcadores, PedidoDeAndamentos, PedidoDoProcesso, ProcessoAberto, Propriedades,
    SessaoDoProcesso, obter_processo,
};
use crate::propriedades_processo::PropriedadesLidas;
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Domain, Processo, configuracao, erro_fonte, saida_erro, transporte};
use serde_json::{Value, json};
use std::fmt::Write as _;
use std::process::ExitCode;

/// As propriedades lidas; ids de assunto e de contato não são publicados.
fn json_das_propriedades(lidas: &PropriedadesLidas) -> Value {
    let assuntos: Vec<Value> = lidas
        .assuntos
        .iter()
        .map(|a| json!({"codigo": a.codigo, "descricao": a.descricao}))
        .collect();
    json!({
        "tipo": lidas.tipo,
        "especificacao": lidas.especificacao,
        "interessados": lidas.interessados,
        "assuntos": assuntos,
        "observacoes": lidas.observacoes,
        "nivel_de_acesso": lidas.nivel_de_acesso.como_texto(),
        "hipotese_legal": lidas.hipotese_legal,
        "data_de_geracao": lidas.data_de_geracao,
        "fonte": lidas.fonte.acao(),
    })
}

/// Os marcadores lidos (sempre lista), o total da legenda e a faixa lida.
fn inserir_marcadores(objeto: &mut serde_json::Map<String, Value>, lidos: &MarcadoresLidos) {
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|m| {
            json!({
                "id": m.id,
                "nome": m.nome,
                "cor": m.cor,
                "texto": m.texto,
                "usuario": m.usuario,
                "data_hora": m.data_hora,
            })
        })
        .collect();
    let faixa = (!itens.is_empty()).then(|| json!({"inicio": 1, "fim": itens.len()}));
    objeto.insert("marcadores".to_owned(), Value::Array(itens));
    objeto.insert(
        "marcadores_total_declarado".to_owned(),
        json!(lidos.total_declarado),
    );
    objeto.insert("marcadores_faixa_observada".to_owned(), json!(faixa));
}

/// Linhas do texto humano dos marcadores: uma por marcador.
fn humano_dos_marcadores(marcadores: &Marcadores, texto: &mut String) {
    let Marcadores::Lidos(lidos) = marcadores else {
        let _ = write!(
            texto,
            "\nMarcadores não lidos ({}).",
            marcadores.motivo().unwrap_or_default()
        );
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
        let _ = write!(texto, "\n- {} [{}]", m.nome, m.cor);
        if let Some(t) = &m.texto {
            let _ = write!(texto, ": {t}");
        }
    }
}

/// Linhas do texto humano das propriedades.
fn humano_das_propriedades(propriedades: &Propriedades, texto: &mut String) {
    let Propriedades::Lidas(lidas) = propriedades else {
        let _ = write!(
            texto,
            "\nPropriedades não lidas ({}).",
            propriedades.motivo().unwrap_or_default()
        );
        return;
    };
    let _ = write!(
        texto,
        "\nPropriedades ({}): tipo {}, nível {}, gerado em {}.",
        lidas.fonte.acao(),
        lidas.tipo.as_deref().unwrap_or("não observado"),
        lidas.nivel_de_acesso.como_texto(),
        lidas
            .data_de_geracao
            .as_deref()
            .unwrap_or("data não observada"),
    );
    if let Some(hipotese) = &lidas.hipotese_legal {
        let _ = write!(texto, "\nHipótese legal: {hipotese}");
    }
    let _ = write!(texto, "\nEspecificação: {}", lidas.especificacao);
    for a in &lidas.assuntos {
        match &a.codigo {
            Some(codigo) => {
                let _ = write!(texto, "\n- assunto {codigo} - {}", a.descricao);
            }
            None => {
                let _ = write!(texto, "\n- assunto {}", a.descricao);
            }
        }
    }
    for interessado in &lidas.interessados {
        let _ = write!(texto, "\n- interessado {interessado}");
    }
    if !lidas.observacoes.is_empty() {
        let _ = write!(texto, "\nObservações: {}", lidas.observacoes);
    }
}

fn json_dos_andamentos(lidos: &AndamentosLidos) -> Value {
    let itens: Vec<Value> = lidos
        .itens
        .iter()
        .map(|a| {
            json!({
                "ocorrido_em": a.ocorrido_em,
                "unidade": a.unidade,
                "usuario": a.usuario,
                "descricao": a.descricao,
            })
        })
        .collect();
    json!({
        "tipo": lidos.tipo.como_texto(),
        "total_declarado": lidos.total_declarado,
        "faixa_observada": {"inicio": 1, "fim": lidos.itens.len()},
        "itens": itens,
    })
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

/// Andamentos, marcadores e propriedades: valor, `null` com motivo ou
/// ausente (omitidos); `nao_lidos` em ordem alfabética.
fn inserir_leituras(
    objeto: &mut serde_json::Map<String, Value>,
    aberto: &ProcessoAberto,
    nao_lidos: &mut Vec<&str>,
) {
    let andamentos = match &aberto.andamentos {
        Andamentos::Lidos(lidos) => Some(json_dos_andamentos(lidos)),
        Andamentos::LinkNaoPublicadoNaArvore => Some(Value::Null),
        Andamentos::Omitidos => None,
    };
    if let Some(valor) = andamentos {
        objeto.insert("andamentos".to_owned(), valor);
    }
    if let Some(motivo) = aberto.andamentos.motivo() {
        nao_lidos.push("andamentos");
        objeto.insert("andamentos_nao_lidos_motivo".to_owned(), json!(motivo));
    }
    match &aberto.marcadores {
        Marcadores::Lidos(lidos) => inserir_marcadores(objeto, lidos),
        Marcadores::NaoPublicadosNaBarra | Marcadores::TabelaNaoPublicada => {
            objeto.insert("marcadores".to_owned(), Value::Null);
        }
        Marcadores::Omitidos => {}
    }
    if let Some(motivo) = aberto.marcadores.motivo() {
        nao_lidos.push("marcadores");
        objeto.insert("marcadores_nao_lidos_motivo".to_owned(), json!(motivo));
    }
    let propriedades = match &aberto.propriedades {
        Propriedades::Lidas(lidas) => Some(json_das_propriedades(lidas)),
        Propriedades::NaoPublicadasNaBarra => Some(Value::Null),
        Propriedades::Omitidas => None,
    };
    if let Some(valor) = propriedades {
        objeto.insert("propriedades".to_owned(), valor);
    }
    if let Some(motivo) = aberto.propriedades.motivo() {
        nao_lidos.push("propriedades");
        objeto.insert("propriedades_nao_lidas_motivo".to_owned(), json!(motivo));
    }
}

/// O JSON publicado; `anexados` sai sempre; `documentos` some com `--sem-documentos`,
/// `andamentos` com `--sem-andamentos`, `propriedades` com `--sem-propriedades`
/// e `marcadores` com `--sem-marcadores` (link não publicado: `null`).
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
        "unidade": aberto.unidade,
        "resolvido_por": aberto.resolvido_por.como_texto(),
        "requisicoes": aberto.requisicoes,
    });
    let mut nao_lidos: Vec<&str> = Vec::new();
    if let Some(objeto) = valor.as_object_mut() {
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
        objeto.insert(
            "anexado_a".to_owned(),
            pai.map_or(
                Value::Null,
                |p| json!({"numero": p.numero, "id_procedimento": p.id_procedimento}),
            ),
        );
    }
    if !sem_documentos && let Some(objeto) = valor.as_object_mut() {
        let documentos: Vec<Value> = aberto
            .arvore
            .documentos
            .iter()
            .map(|d| {
                json!({
                    "id_documento": d.id,
                    "rotulo": d.rotulo,
                    "numero_sei": d.numero_sei,
                    "pasta": d.pasta,
                })
            })
            .collect();
        objeto.insert("documentos".to_owned(), Value::Array(documentos));
        let movidos: Vec<Value> = aberto
            .arvore
            .documentos_movidos
            .iter()
            .map(|m| {
                json!({
                    "id": m.id,
                    "rotulo": m.rotulo,
                    "numero_sei": m.numero_sei,
                    "destino": m.destino.as_ref().map(
                        |d| json!({"numero": d.numero, "id_procedimento": d.id_procedimento}),
                    ),
                })
            })
            .collect();
        objeto.insert("documentos_movidos".to_owned(), Value::Array(movidos));
        let pastas: Vec<Value> = aberto
            .arvore
            .pastas
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "rotulo": p.rotulo,
                    "total": p.total_declarado,
                    "verificacao": p.verificacao.map(VerificacaoDaPasta::como_texto),
                    "conferidos_por_identidade": p.conferidos_por_identidade,
                    "conferidos_por_contagem": p.conferidos_por_contagem,
                })
            })
            .collect();
        objeto.insert("pastas".to_owned(), Value::Array(pastas));
        let verificacao = verificacao_das_pastas(&aberto.arvore.pastas);
        objeto.insert(
            "verificacao_das_pastas".to_owned(),
            json!(verificacao.map(VerificacaoDaPasta::como_texto)),
        );
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
        let _ = write!(
            texto,
            "\n{} documento(s) na árvore, {} pasta(s){}:",
            arvore.documentos.len(),
            arvore.pastas.len(),
            verificacao_das_pastas(&arvore.pastas)
                .map(|v| format!(" conferidas {}", v.como_texto()))
                .unwrap_or_default()
        );
        for documento in &arvore.documentos {
            let _ = write!(texto, "\n- {} (id {})", documento.rotulo, documento.id);
            let pasta = documento
                .pasta
                .as_ref()
                .and_then(|id| arvore.pastas.iter().find(|p| &p.id == id));
            if let Some(pasta) = pasta {
                let _ = write!(texto, " [pasta {}]", pasta.rotulo);
            }
        }
        humano_dos_movidos(&arvore.documentos_movidos, &mut texto);
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
                    a.ocorrido_em.as_deref().unwrap_or("?"),
                    a.unidade.as_deref().unwrap_or("?"),
                    a.descricao
                );
            }
        }
        outro => {
            let _ = write!(
                texto,
                "\nAndamentos não lidos ({}).",
                outro.motivo().unwrap_or_default()
            );
        }
    }
    humano_das_propriedades(&aberto.propriedades, &mut texto);
    humano_dos_marcadores(&aberto.marcadores, &mut texto);
    texto
}

fn humano_dos_movidos(movidos: &[DocumentoMovido], texto: &mut String) {
    if movidos.is_empty() {
        return;
    }
    let _ = write!(
        texto,
        "\n{} documento(s) movido(s) para outro processo:",
        movidos.len()
    );
    for m in movidos {
        let _ = write!(texto, "\n- {} (id {}): ", m.rotulo, m.id);
        match &m.destino {
            Some(d) => {
                let _ = write!(texto, "movido para {}, ies processo {}", d.numero, d.numero);
            }
            None => texto.push_str("destino não publicado"),
        }
    }
}

fn erro_de_uso(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn obter(cli: &Cli, comando: &Processo, nup: &NumeroProcesso) -> ExitCode {
    if comando.andamentos_completos && comando.sem_andamentos {
        return erro_de_uso(
            cli.json,
            "sem_andamentos_incompativel_com_andamentos_completos",
            "--sem-andamentos e --andamentos-completos são incompatíveis",
        );
    }
    let andamentos = if comando.sem_andamentos {
        PedidoDeAndamentos::Omitidos
    } else if comando.andamentos_completos {
        PedidoDeAndamentos::Completos
    } else {
        PedidoDeAndamentos::VisaoServida
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    let lidos = transporte::arquivo_privado(&arquivos.url_controle, 8_192).and_then(|url| {
        transporte::arquivo_privado(&arquivos.origem, 512).map(|origem| (url, origem))
    });
    let (url, origem) = match lidos {
        Ok(lidos) => lidos,
        Err(falha) => return erro_fonte(cli.json, falha.subcodigo(), falha.mensagem()),
    };
    let sessao = SessaoDoProcesso {
        url_do_controle: url.trim(),
        origem: origem.trim(),
        cookie_file: &arquivos.cookie,
    };
    let pedido = PedidoDoProcesso {
        nup,
        unidade: cli.unidade.as_deref(),
        andamentos,
        propriedades: !comando.sem_propriedades,
        marcadores: !comando.sem_marcadores,
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    match obter_processo(&sessao, &pedido, &indice, &mut EnvioHttps) {
        Ok(aberto) => {
            if cli.json {
                let valor = json_do_processo(nup, &aberto, comando.sem_documentos);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&valor).unwrap_or_default()
                );
            } else {
                println!("{}", humano(nup, &aberto, comando.sem_documentos));
            }
            ExitCode::SUCCESS
        }
        Err(erro) => saida_erro::erro_fonte_http(
            cli.json,
            erro.subcodigo(),
            erro.mensagem(),
            erro.status_http(),
        ),
    }
}

/// `Some` só para `ies processo NUP` sem `--resumo` nem fonte offline; NUP
/// inválido fica para a validação comum (saída 2).
pub(crate) fn tentar_obter(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(
        comando @ Processo {
            numero: Some(numero),
            resumo: false,
            historico_privado: None,
            resultado_pesquisa_file: None,
            acao: None,
            ..
        },
    ) = &cli.command
    else {
        return None;
    };
    let nup = NumeroProcesso::novo(numero).ok()?;
    Some(obter(cli, comando, &nup))
}

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
