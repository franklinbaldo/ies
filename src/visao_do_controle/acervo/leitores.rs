//! Cada resposta de `procedimento_controlar` pelo caminho da CLI: o
//! transporte (status, MIME, limite, charset, tela de login), a visão, a
//! unidade e, conforme a visão, os leitores R ou D. A unidade "solicitada"
//! sai da gravação: a da tela do Controle anterior na mesma cassette.
use super::super::passos::{contexto, unidade_da_tela};
use super::{Leitor, Lista, PaginaLida, Registro, Replay, Resultado, paginacao};
use crate::cassette::acervo::gravacao::{self, Resposta};
use crate::controle_contexto::ContextoPagina;
use crate::dominio::Categoria;
use crate::erro_html::ErroHtml;
use crate::erro_publico::Rotulo as _;
use crate::transporte::{FalhaTransporte, RespostaWeb, decodificar_resposta};
use crate::unidade_categoria::{CategoriaLista, selecionar};
use base64::Engine as _;

/// O que vale para todas as respostas de uma cassette.
pub(super) struct Cassette<'a> {
    pub(super) card: &'a str,
    pub(super) instalacao: &'a str,
}

impl Cassette<'_> {
    pub(super) fn registro(&self, leitor: Leitor, passo: usize, resultado: Resultado) -> Registro {
        Registro {
            leitor,
            instalacao: self.instalacao.to_owned(),
            card: self.card.to_owned(),
            passo,
            resultado,
            contagens: Vec::new(),
            formas: Vec::new(),
        }
    }

    pub(super) fn pagina(&self, passo: usize, lista: Lista, unidade: &str) -> PaginaLida {
        PaginaLida {
            card: self.card.to_owned(),
            instalacao: self.instalacao.to_owned(),
            passo,
            lista,
            unidade: unidade.to_owned(),
            criterio: None,
            total: 0,
            faixa: None,
            ids: Vec::new(),
        }
    }
}

pub(super) fn replay(card: &str, instalacao: &str, json: &serde_json::Value) -> Replay {
    let cas = Cassette { card, instalacao };
    let mut saida = Replay::default();
    let mut anterior: Option<String> = None;
    for resp in gravacao::respostas(json) {
        let pathname = resp.interacao["request"]["pathname"].as_str().unwrap_or("");
        if resp.metodo == "GET" && resp.status == 200 && pathname.ends_with("/sip/login.php") {
            saida.registros.push(tela_de_login(&cas, &resp));
        }
        if resp.operacao == "procedimento_controlar" {
            ler(&cas, &resp, &mut anterior, &mut saida);
        }
    }
    saida
}

fn bytes(interacao: &serde_json::Value) -> Vec<u8> {
    interacao["response"]["body_base64"]
        .as_str()
        .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        .unwrap_or_default()
}

fn mime(interacao: &serde_json::Value) -> String {
    interacao["response"]["headers"]
        .as_object()
        .and_then(|h| {
            h.iter()
                .find(|(nome, _)| nome.eq_ignore_ascii_case("content-type"))
                .and_then(|(_, v)| v.as_str())
        })
        .unwrap_or("")
        .to_owned()
}

/// A mesma decodificação do GET do Controle e do POST do formulário.
pub(super) fn transporte(resp: &Resposta<'_>) -> Result<String, FalhaTransporte> {
    let corpo = bytes(resp.interacao);
    decodificar_resposta(RespostaWeb {
        status_code: i32::try_from(resp.status).unwrap_or(0),
        mime: mime(resp.interacao),
        content_length: None,
        localizacao: None,
        corpo: corpo.as_slice(),
    })
}

fn tela_de_login(cas: &Cassette<'_>, resp: &Resposta<'_>) -> Registro {
    let resultado = match transporte(resp) {
        Ok(_) => Resultado::Lido,
        Err(falha) => Resultado::Recusado(falha.rotulo()),
    };
    cas.registro(Leitor::TelaDeLogin, resp.passo, resultado)
}

fn rotulo_do_transporte(resp: &Resposta<'_>, falha: &FalhaTransporte) -> &'static str {
    // No POST, a tela de login vira a recusa do envio do formulário.
    if resp.metodo == "POST" && *falha == FalhaTransporte::PaginaDeLoginRecebidaNoControle {
        return crate::formulario_web::ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio
            .rotulo();
    }
    falha.rotulo()
}

fn ler(cas: &Cassette<'_>, resp: &Resposta<'_>, anterior: &mut Option<String>, saida: &mut Replay) {
    let mut t = cas.registro(Leitor::Transporte, resp.passo, Resultado::Lido);
    t.formas.push(format!(
        "{}_{}",
        resp.metodo.to_ascii_lowercase(),
        resp.status
    ));
    let html = match transporte(resp) {
        Ok(html) => html,
        Err(falha) => {
            t.resultado = Resultado::Recusado(rotulo_do_transporte(resp, &falha));
            if let Some(local) = gravacao::localizacao(resp.interacao) {
                let acao = parametro_de(local, "acao").unwrap_or("outra");
                t.formas.push(format!("location_acao_{acao}"));
            }
            saida.registros.push(t);
            return;
        }
    };
    saida.registros.push(t);
    let mut v = cas.registro(Leitor::Visao, resp.passo, Resultado::Lido);
    let ctx = match contexto(&html) {
        Ok(ctx) => ctx,
        Err(erro) => {
            v.resultado = Resultado::Recusado(erro.rotulo());
            saida.registros.push(v);
            return;
        }
    };
    let visao = if ctx.eh_detalhada() {
        "D"
    } else if ctx.eh_resumida() {
        "R"
    } else {
        v.resultado = Resultado::Recusado(
            crate::visao_do_controle::ErroVisaoDoControle::Html(ErroHtml::valor(
                "visao_do_controle_nao_reconhecida",
            ))
            .rotulo(),
        );
        "ausente"
    };
    v.formas.push(format!("visao_{visao}"));
    let filtro = gravacao::parametro(resp.interacao, "tipo_filtro");
    if let Some(f) = filtro.first() {
        v.formas.push(format!("tipo_filtro_{f}"));
    }
    saida.registros.push(v);
    let mut u = cas.registro(Leitor::Unidade, resp.passo, Resultado::Lido);
    let unidade = match unidade_da_tela(&ctx, None) {
        Ok(unidade) => unidade,
        Err(erro) => {
            u.resultado = Resultado::Recusado(erro.rotulo());
            saida.registros.push(u);
            return;
        }
    };
    saida.registros.push(u);
    if let Some(prev) = anterior.replace(unidade.clone()) {
        saida
            .registros
            .push(solicitada(cas, resp, &html, &ctx, &prev, &unidade));
    }
    if visao == "ausente" {
        return;
    }
    // Filtro na URL servida: o comando lê a URL do Controle guardada.
    let fora = (!filtro.is_empty()).then_some("filtro_na_url_servida");
    if ctx.eh_resumida() {
        resumida(cas, resp, &html, &unidade, fora, saida);
    } else {
        paginacao::detalhada(cas, resp, &html, &unidade, fora, saida);
    }
}

fn parametro_de<'a>(url: &'a str, nome: &str) -> Option<&'a str> {
    let consulta = url.split_once('?')?.1;
    consulta
        .split('&')
        .filter_map(|par| par.split_once('='))
        .find(|(chave, _)| *chave == nome)
        .map(|(_, valor)| valor)
}

/// `--unidade` igual à tela anterior da cassette: a mesma unidade é lida;
/// outra (troca de unidade gravada) é recusada, na R e na D.
fn solicitada(
    cas: &Cassette<'_>,
    resp: &Resposta<'_>,
    html: &str,
    ctx: &ContextoPagina,
    prev: &str,
    unidade: &str,
) -> Registro {
    let mut s = cas.registro(Leitor::UnidadeSolicitada, resp.passo, Resultado::Lido);
    s.formas.push(
        if prev == unidade {
            "mesma_unidade"
        } else {
            "outra_unidade"
        }
        .into(),
    );
    if let Err(erro) = unidade_da_tela(ctx, Some(prev)) {
        s.resultado = Resultado::Recusado(erro.rotulo());
    }
    // A lista R aplica a mesma conferência (`unidade_inesperada`), quando lida.
    let pagina = [html.to_owned()];
    if ctx.eh_resumida() && crate::paginas::juntar(&pagina, None).is_ok() {
        let juntar = crate::paginas::juntar(&pagina, Some(prev));
        if juntar.is_ok() != (s.resultado == Resultado::Lido) {
            s.formas.push("juntar_diverge_da_tela".into());
        }
    }
    s
}

fn resumida(
    cas: &Cassette<'_>,
    resp: &Resposta<'_>,
    html: &str,
    unidade: &str,
    fora: Option<&'static str>,
    saida: &mut Replay,
) {
    for leitor in [Leitor::Detalhada, Leitor::PaginaDetalhada] {
        let motivo = Resultado::NaoAplicavel("tela_na_visao_r_o_comando_troca_para_d");
        saida
            .registros
            .push(cas.registro(leitor, resp.passo, motivo));
    }
    let mut j = cas.registro(Leitor::Resumida, resp.passo, Resultado::Lido);
    let leitura = match crate::paginas::juntar(&[html.to_owned()], None) {
        Ok(leitura) => leitura,
        Err(erro) => {
            j.resultado = fora.map_or(Resultado::Recusado(erro.rotulo()), Resultado::NaoAplicavel);
            j.formas.push(format!("recusa_{}", erro.rotulo()));
            saida.registros.push(j);
            for leitor in [Leitor::Recebidos, Leitor::Gerados, Leitor::LinhasComLink] {
                let motivo = Resultado::NaoAplicavel("lista_r_nao_lida");
                saida
                    .registros
                    .push(cas.registro(leitor, resp.passo, motivo));
            }
            return;
        }
    };
    j.contagens = vec![
        ("recebidos", leitura.caixa.recebidos.processos.len() as u64),
        ("gerados", leitura.caixa.gerados.processos.len() as u64),
    ];
    saida.registros.push(j);
    for (leitor, lista, categoria) in [
        (
            Leitor::Recebidos,
            CategoriaLista::Recebidos,
            Lista::Recebidos,
        ),
        (Leitor::Gerados, CategoriaLista::Gerados, Lista::Gerados),
    ] {
        let mut l = cas.registro(leitor, resp.passo, Resultado::Lido);
        match selecionar(&leitura, lista, "acervo") {
            Ok(modelo) => {
                let processos = modelo["processos"].as_array().map_or(0, Vec::len) as u64;
                let total = modelo["total_declarado"].as_u64().unwrap_or(0);
                l.contagens = vec![("linhas", processos), ("total_declarado", total)];
                l.formas.push(forma_da_lista(processos, total).into());
                let faixa = &modelo["faixa_observada"];
                let faixa = faixa["inicio"].as_u64().zip(faixa["fim"].as_u64());
                if let Some((i, f)) = faixa {
                    l.contagens.push(("faixa_inicio", i));
                    l.contagens.push(("faixa_fim", f));
                }
                let tabela = match lista.categoria() {
                    Categoria::Recebidos => &leitura.caixa.recebidos,
                    Categoria::Gerados => &leitura.caixa.gerados,
                };
                let mut p = cas.pagina(resp.passo, categoria, unidade);
                p.total = usize::try_from(total).unwrap_or(usize::MAX);
                p.faixa = faixa.map(|(i, f)| {
                    (
                        usize::try_from(i).unwrap_or(0),
                        usize::try_from(f).unwrap_or(0),
                    )
                });
                p.ids = tabela
                    .processos
                    .iter()
                    .map(|x| x.id.as_str().to_owned())
                    .collect();
                p.criterio = paginacao::criterio(html, lista.categoria());
                saida.paginas.push(p);
            }
            Err(erro) => {
                l.resultado = Resultado::Recusado(crate::erro_publico::Rotulo::rotulo(&erro));
            }
        }
        saida.registros.push(l);
        saida
            .registros
            .push(paginacao::da_tabela(cas, resp, html, lista.categoria()));
    }
    let mut k = cas.registro(Leitor::LinhasComLink, resp.passo, Resultado::Lido);
    match crate::caixa::linhas_com_link(html) {
        Ok(linhas) => {
            let com = linhas.iter().filter(|l| l.href.is_some()).count() as u64;
            k.contagens = vec![("linhas", linhas.len() as u64), ("com_href", com)];
        }
        Err(erro) => k.resultado = Resultado::Recusado(erro.rotulo()),
    }
    saida.registros.push(k);
}

pub(super) fn forma_da_lista(linhas: u64, total: u64) -> &'static str {
    if total == 0 {
        "vazia"
    } else if linhas == total {
        "uma_pagina"
    } else {
        "paginada"
    }
}
