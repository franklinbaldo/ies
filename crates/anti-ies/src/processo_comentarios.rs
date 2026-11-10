//! Lista de comentários simulada a partir da fotografia SQLite explícita.
//! A forma HTML é hipótese local; nunca prova paridade com a cassette original.
use crate::{Estado, Resposta, eh_controlador_sei, escapar, parametro_unico};
use iesei::comentarios::{Comentario, Observacao};
use iesei::instalacao::Instalacao;
use rusqlite::Connection;

fn linha(item: &Comentario, com_texto_na_coluna: bool) -> String {
    let id = escapar(&item.id);
    let data = escapar(&item.data_hora);
    let unidade = escapar(&item.unidade);
    let usuario = escapar(&item.usuario);
    let texto = escapar(&item.texto);
    if com_texto_na_coluna {
        format!(
            "<tr><td><input type='checkbox' value='{id}'></td><td>{data}</td><td>{unidade}</td><td>{usuario}</td><td>{}</td><td>{}</td><td>{texto}</td><td></td></tr>",
            escapar(item.protocolo.as_deref().unwrap_or("")),
            escapar(item.tipo.as_deref().unwrap_or("")),
        )
    } else {
        format!(
            "<tr><td><input type='checkbox' value='{id}'></td><td>{data}</td><td>{unidade}</td><td>{usuario}</td><td></td></tr><tr class='trComentario'><td colspan='5'>{texto}</td></tr>"
        )
    }
}

fn tabela(id: &str, observacao: &Observacao) -> String {
    let n = observacao.total_declarado;
    let forma = if n == 1 { "registro" } else { "registros" };
    let visao = match observacao.visao {
        iesei::comentarios::Visao::Processo => "P",
        iesei::comentarios::Visao::Todos => "T",
    };
    let com_texto_na_coluna = observacao.itens.iter().all(|item| item.protocolo.is_some())
        && !observacao.itens.is_empty();
    let cabecalho = if com_texto_na_coluna {
        "<th></th><th>Data</th><th>Unidade</th><th>Usuário</th><th>Protocolo</th><th>Tipo</th><th>Texto</th><th>Ações</th>"
    } else {
        "<th></th><th>Data</th><th>Unidade</th><th>Usuário</th><th>Ações</th>"
    };
    let mut linhas = String::new();
    for item in &observacao.itens {
        linhas.push_str(&linha(item, com_texto_na_coluna));
    }
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    format!(
        "<html><body><form id='frmComentarioLista'><input type='hidden' name='hdnVisualizacao' value='{visao}'><table id='tblComentarios'><caption>Lista de Comentários ({n} {forma}):</caption><thead><tr>{cabecalho}</tr></thead><tbody>{linhas}</tbody></table><a href='controlador.php?acao=comentario_cadastrar&amp;id_procedimento={id}&amp;infra_hash={hash}'>Novo</a></form></body></html>"
    )
}

fn pagina_cadastro(id: &str) -> Resposta {
    let hash = crate::protocolo_sintetico::INFRA_HASH;
    Resposta::html(format!(
        "<html><body><form id='frmComentarioCadastro' method='post' action='controlador.php?acao=comentario_cadastrar&amp;id_procedimento={id}&amp;infra_hash={hash}'><textarea name='txaDescricao'></textarea><input type='submit' name='sbmCadastrarComentario' value='Cadastrar'></form></body></html>"
    ))
}


/// Hipótese de confirmação: POST único no formulário servido e redirect
/// para lista com identidade do aceite. A autoria/data são sintéticas.
fn gravar(
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let Some(pares) = crate::processo_envio::pares(corpo) else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let campo = |nome| crate::processo_envio::campo_unico(&pares, nome);
    let Some(texto) = campo("txaDescricao").and_then(crate::processo_envio::decodificar_formulario)
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if pares.len() != 2 || campo("sbmCadastrarComentario") != Some("Cadastrar")
        || texto.trim().is_empty()
    {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Ok(Some(foto)) = iesei::comentarios::ler(banco, origem, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Some(anterior) = foto.itens.first() else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let novo = Comentario {
        id: String::new(),
        data_hora: "10/11/2026 09:00".to_owned(),
        unidade: anterior.unidade.clone(),
        usuario: anterior.usuario.clone(),
        texto,
        protocolo: None,
        tipo: None,
    };
    match iesei::comentarios::acrescentar(banco, origem, cenario, id, novo) {
        Ok(id_aceito) => Resposta::redirecionar(format!(
            "controlador.php?acao=comentario_listar&id_procedimento={id}&id_comentario={id_aceito}"
        )),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !eh_controlador_sei(alvo)
        || !matches!(
            parametro_unico(alvo, "acao"),
            Some("comentario_listar" | "comentario_cadastrar")
        )
    {
        return Resposta::nova(Estado::NaoEncontrado);
    }
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let cadastro = parametro_unico(alvo, "acao") == Some("comentario_cadastrar");
    if metodo == "POST" {
        return if cadastro {
            gravar(corpo, banco, origem, cenario, id)
        } else {
            Resposta::nova(Estado::MetodoNaoPermitido)
        };
    }
    match iesei::comentarios::ler(banco, origem, cenario, id) {
        Ok(Some(_)) if cadastro => pagina_cadastro(id),
        Ok(Some(foto)) if foto.itens.is_empty() => {
            // Sem comentários o SEI pode redirecionar ao cadastro; a CLI
            // não interpreta esse 302 como uma lista vazia comprovada.
            Resposta::redirecionar(format!(
                "controlador.php?acao=comentario_cadastrar&id_procedimento={id}&infra_hash={}",
                crate::protocolo_sintetico::INFRA_HASH,
            ))
        }
        Ok(Some(foto)) => Resposta::html(tabela(id, &foto)),
        Ok(None) | Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}
