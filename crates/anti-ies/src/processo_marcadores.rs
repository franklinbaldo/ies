//! Marcadores projetados do SQLite efêmero: GET e remoção local confirmável.
//! O POST opera exclusivamente a associação simulada, nunca o SEI externo.
//! BDD-255-01 (catálogo separado), BDD-255-02 (remoção) e BDD-255-03.
use crate::{Estado, Resposta, documentos::campo_formulario_unico, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::marcadores::{Marcador, Observacao};
use rusqlite::Connection;
use std::fmt::Write as _;

fn cabecalho(html: &mut String, nup: &str, id: &str, observacao: &Observacao) {
    let ids = observacao
        .itens
        .iter()
        .map(|m| m.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let nup = escapar(nup);
    let assinatura = crate::protocolo_sintetico::INFRA_HASH;
    write!(
        html,
        "<!doctype html><html><head><title>SEI - Marcadores do Processo {nup}</title></head><body>\
         <form id='frmGerenciarMarcador' method='post' \
         action='controlador.php?acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&infra_hash={assinatura}'>\
         <input type='hidden' id='hdnInfraTipoPagina' name='hdnInfraTipoPagina' value='2'>\
         <input type='hidden' id='hdnInfraNroItens' name='hdnInfraNroItens' value='{}'>\
         <input type='hidden' id='hdnInfraItemId' name='hdnInfraItemId' value=''>\
         <input type='hidden' id='hdnInfraItens' name='hdnInfraItens' value='{}'>\
         <input type='hidden' id='hdnInfraItensHash' name='hdnInfraItensHash' value='{}'>\
         <button type='button' id='btnAdicionar' onclick=\"location.href='controlador.php?acao=andamento_marcador_cadastrar&amp;id_procedimento={id}&amp;infra_hash={assinatura}'\">Adicionar</button>\
         <button type='button' id='btnRemover' onclick='acaoRemocaoMultipla()'>Remover</button>\
         <table id='tblMarcadores' summary='Marcadores'>\
         <caption>Marcadores ({} {}):</caption>\
         <tr><th><a id='lnkInfraCheck'></a></th>\
         <th><div class='infraDivRotuloOrdenacao'>Marcador</div></th>\
         <th>Texto</th>\
         <th><div class='infraDivRotuloOrdenacao'>Usuário</div></th>\
         <th><div class='infraDivRotuloOrdenacao'>Data/Hora</div></th>\
         <th>Ações</th></tr>",
        observacao.itens.len(),
        escapar(&ids),
        assinatura,
        observacao.total_declarado,
        if observacao.total_declarado == 1 { "registro" } else { "registros" }
    ).expect("String");
}

// Preservar o argumento literal do handler observado sem injeção de JS ou HTML.
fn argumento_js(texto: &str) -> String {
    let mut saida = String::new();
    for c in texto.chars() {
        match c {
            '\\' => saida.push_str("\\\\"),
            '\'' => saida.push_str("\\'"),
            '\n' => saida.push_str("\\n"),
            '\r' => saida.push_str("\\r"),
            '\u{2028}' => saida.push_str("\\u2028"),
            '\u{2029}' => saida.push_str("\\u2029"),
            c if c.is_control() => write!(saida, "\\u{:04x}", c as u32).expect("String"),
            c => saida.push(c),
        }
    }
    escapar(&saida)
}

fn linha(html: &mut String, m: &Marcador) {
    let nome_js = argumento_js(&m.nome);
    let (id, nome, cor, data) = (
        escapar(&m.id),
        escapar(&m.nome),
        escapar(&m.cor),
        escapar(&m.data_hora),
    );
    let texto = m.texto.as_deref().map(escapar).unwrap_or_default();
    let usuario = m.usuario.as_deref().map_or_else(String::new, |nome| {
        let nome = escapar(nome);
        format!("<a class='ancoraSigla'>{nome}</a>")
    });
    write!(
        html,
        "<tr class='infraTrClara'>\
         <td><a id='lnkInfraID-{id}' name='ID-{id}'></a>\
         <input type='checkbox' class='infraCheckbox' value='{id}' title='{nome}'></td>\
         <td><img src='svg/marcador_{cor}.svg' title='{nome}'>{nome}</td>\
         <td id='tdAndamentoMarcador{id}'>{texto}</td>\
         <td>{usuario}</td><td>{data}</td>\
         <td><a href='#' onclick=\"acaoRemover('{id}','{nome_js}');return false;\">Remover</a></td></tr>"
    )
    .expect("String");
}

/// Cadastro aberto a partir do GET de gerenciamento: catálogo vem da unidade,
/// nunca da associação existente no processo. Nenhum POST é presumido.
fn formulario_adicionar(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Resposta {
    let Ok(Some(opcoes)) = iesei::catalogo_marcadores::ler(banco, origem, cenario, unidade) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    let mut html = format!(
        "<!doctype html><html><body><h1>Adicionar Marcador</h1>\
         <form id='frmAndamentoMarcadorCadastro' method='post' \
         action='controlador.php?acao=andamento_marcador_cadastrar&acao_origem=andamento_marcador_cadastrar&arvore=1&id_procedimento={id}&infra_hash={hash}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
         <button type='submit' id='sbmSalvar' name='sbmSalvar' value='Salvar'>Salvar</button>\
         <button type='button' id='btnVoltar' name='btnVoltar'>Voltar</button>\
         <select id='selMarcador' name='selMarcador'>\
         <option value='null' selected='selected'>&nbsp;</option>"
    );
    for opcao in opcoes {
        write!(
            html,
            "<option value='{}' data-imagesrc='svg/marcador_{}.svg'>{}</option>",
            escapar(&opcao.id),
            escapar(&opcao.cor),
            escapar(&opcao.nome)
        )
        .expect("String");
    }
    write!(
        html,
        "</select><input type='hidden' id='hdnIdMarcador' name='hdnIdMarcador' value=''>\
         <textarea id='txaTexto' name='txaTexto' rows='4'></textarea>\
         <input type='hidden' name='hdnIdProtocolo' value='{id}'>\
         </form></body></html>"
    )
    .expect("String");
    Resposta::html(html)
}

/// Interpreta uma única submissão de catálogo, exclusivamente na bancada SQLite.
fn submeter_atribuicao(
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
) -> Resposta {
    let (Some(tipo), Some(marcador), Some(protocolo), Some(texto), Some(salvar)) = (
        campo_formulario_unico(corpo, "hdnInfraTipoPagina"),
        campo_formulario_unico(corpo, "hdnIdMarcador"),
        campo_formulario_unico(corpo, "hdnIdProtocolo"),
        campo_formulario_unico(corpo, "txaTexto"),
        campo_formulario_unico(corpo, "sbmSalvar"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if corpo.split('&').count() != 5 || tipo != "2" || protocolo != id || salvar != "Salvar" {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match iesei::marcadores::atribuir(banco, origem, cenario, id, unidade, &marcador, &texto) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_cadastrar&id_procedimento={id}"
        )),
        Err(iesei::marcadores::ErroMarcadores::DadoInvalido) => {
            Resposta::nova(Estado::RequisicaoInvalida)
        }
        Err(
            iesei::marcadores::ErroMarcadores::Sqlite(_)
            | iesei::marcadores::ErroMarcadores::ProcessoNaoObservado,
        ) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

/// Retirada única validada contra a observação íntegra, apenas no SQLite local.
fn submeter_retirada(
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
    unidade: &str,
    observacao: &Observacao,
) -> Resposta {
    let (Some(tipo), Some(nro), Some(item), Some(itens), Some(hash)) = (
        campo_formulario_unico(corpo, "hdnInfraTipoPagina"),
        campo_formulario_unico(corpo, "hdnInfraNroItens"),
        campo_formulario_unico(corpo, "hdnInfraItemId"),
        campo_formulario_unico(corpo, "hdnInfraItens"),
        campo_formulario_unico(corpo, "hdnInfraItensHash"),
    ) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let ids = observacao
        .itens
        .iter()
        .map(|m| m.id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    if corpo.split('&').count() != 5
        || tipo != "2"
        || nro != observacao.total_declarado.to_string()
        || itens != ids
        || hash != crate::protocolo_sintetico::INFRA_HASH
        || !observacao.itens.iter().any(|m| m.id == item)
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    match iesei::marcadores::retirar(banco, origem, cenario, id, unidade, &item) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=andamento_marcador_gerenciar&acao_origem=andamento_marcador_remover&id_procedimento={id}"
        )),
        Err(iesei::marcadores::ErroMarcadores::DadoInvalido) => {
            Resposta::nova(Estado::RequisicaoInvalida)
        }
        Err(
            iesei::marcadores::ErroMarcadores::Sqlite(_)
            | iesei::marcadores::ErroMarcadores::ProcessoNaoObservado,
        ) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

/// Responde à tela de leitura cuja ação foi publicada na barra da árvore.
/// Sem observação não há afirmação de lista vazia nem rota de escrita.
///
/// # Cautela
/// O HTML local é hipótese estrutural, não transcrição de uma cassette.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let acao = parametro_unico(alvo, "acao");
    if !matches!(
        (metodo, acao),
        (
            "GET",
            Some("andamento_marcador_gerenciar" | "andamento_marcador_cadastrar")
        ) | (
            "POST",
            Some("andamento_marcador_remover" | "andamento_marcador_cadastrar")
        )
    ) {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(processo)) = iesei::processos::ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let unidade = match crate::processo_envio::unidade_da_sessao(banco, origem, cenario, &processo)
    {
        Ok(unidade) => unidade,
        Err(estado) => return Resposta::nova(estado),
    };
    if !processo.aberto_em.contains(&unidade) {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(observacao)) = iesei::marcadores::ler(banco, origem, cenario, id, &unidade) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if metodo == "GET" && acao == Some("andamento_marcador_cadastrar") {
        return formulario_adicionar(banco, origem, cenario, id, &unidade);
    }
    if metodo == "POST" && acao == Some("andamento_marcador_cadastrar") {
        return submeter_atribuicao(corpo, banco, origem, cenario, id, &unidade);
    }
    if metodo == "POST" {
        return submeter_retirada(corpo, banco, origem, cenario, id, &unidade, &observacao);
    }
    // UnB 4.0.9: sem marcador, GET redireciona ao formulário de adicionar.
    // Não inventar "0 registros" na tela de gerenciamento.
    if observacao.itens.is_empty() {
        return Resposta::redirecionar(format!(
            "controlador.php?acao=andamento_marcador_cadastrar&acao_origem=andamento_marcador_gerenciar&id_procedimento={id}&infra_hash={}",
            crate::protocolo_sintetico::INFRA_HASH
        ));
    }
    let mut html = String::new();
    cabecalho(&mut html, &processo.numero, id, &observacao);
    for marcador in &observacao.itens {
        linha(&mut html, marcador);
    }
    html.push_str("</table></form>");
    // O action vem literal do handler servido, não é criado pela CLI.
    let assinado = crate::protocolo_sintetico::INFRA_HASH;
    write!(
        html,
        "<script>function acaoRemover(id,desc){{\
         document.getElementById('hdnInfraItemId').value=id;\
         document.getElementById('frmGerenciarMarcador').action='controlador.php?acao=andamento_marcador_remover&acao_origem=andamento_marcador_gerenciar&acao_retorno=andamento_marcador_gerenciar&id_procedimento={id}&arvore=1&infra_hash={assinado}';\
         document.getElementById('frmGerenciarMarcador').submit();\
         }}</script></body></html>"
    )
    .expect("String");
    Resposta::html(html)
}
