---
type: CommandContract
title: Cancelar documento pela barra Web, com prévia e verificação da árvore
operation: documento.cancelar
command: ies documento cancelar DOC --motivo TEXTO [--confirmar]
effect: write-document-cancellation
implementation: partial
---

# `ies documento cancelar DOC --motivo TEXTO [--confirmar]` (Refs #273)

Cancela irreversivelmente um documento por sua ação **publicada na barra do nó** `DOCUMENTO`. Só a interface Web do SEI, sem inventar URL, sem executar JavaScript e sem REST. A CLI nunca altera o estado sem `--confirmar`. Escrita ao vivo é permitida **somente em instalações de treinamento**, mediante prévia e verificação por leitura; nunca produção.

## Contrato de entrada

- `DOC` é o número SEI validado como em `ies documento DOC`. `--processo NUP` opcional usa a mesma abertura até a árvore, sem GET do conteúdo.
- `--motivo TEXTO` **obrigatório**, não vazio após `trim`, representável em ISO-8859-1 e sem controles. Uso inválido sai com 2 **antes de qualquer acesso à sessão**.
- `--confirmar` opcional. Sem ele há apenas prévia; com ele executa **um POST** e jamais retry.
- `--json` e `--unidade` seguem as opções globais.

## Ação e formulário

1. Abre o documento até a árvore, confere NUP, id e número SEI. Exige `natureza.cancelado == false`. Se já cancelado, recusa sem abrir a tela. Se estado não observado, não tenta escrever.
2. Da única barra `Nos[i].acoes` **desse documento** obtém a única âncora cujo href traz `acao=documento_cancelar`; ausência recusa `cancelamento_nao_oferecido_na_barra`. Confere o endereço relativo contra a instalação, `id_procedimento`, `id_documento` e `infra_hash` único (64 hex). Duplicata, outro documento ou URL suspeita são recusados.
3. Faz GET da tela publicada; 302 não é seguido. Confere `form#frmDocumentoCancelar`, `method=post`, `action` do mesmo processo e documento, um `textarea name=txaMotivo` habilitado, o oculto `hdnInfraTipoPagina` e único `button type=submit name=sbmSalvar`. Não aceita campos enviáveis adicionais.
4. Deriva a submissão pelo primitivo compartilhado `formulario_web`, com `EfeitoAvaliado::AlteraEstadoDoProcesso { CancelamentoDoDocumento }`, `txaMotivo` e o botão `sbmSalvar`; `charset` da tela.
5. **Prévia** publica identidade, motivo, `confirmado:false`, `alterado:false` e contagem, sem POST. **Confirmação** faz um único POST. A resposta esperada é 302 sem seguir para `arvore_visualizar` do mesmo processo/documento, com `atualizar_arvore=1`. Mesmo 5xx, login, rede ou 302 inesperado **não** autorizam retry.
6. Sempre uma nova leitura da árvore após qualquer envio possivelmente feito; só considera sucesso o ícone `documento_cancelado` juntamente da dica `Documento Cancelado: <motivo>` igual ao solicitado. Se não houve cancelamento demonstrado, saída 1; leitura falha ou efeito ambíguo, saída 3.

## Contagem de requisições

Abertura do documento, pelo `--processo` a frio: 3 GET. Prévia: +1 GET da tela (4). `--confirmar`: +1 POST e +1 GET para verificação (6). Caminhos aquecidos e pesquisa rápida têm custos próprios da abertura; cada tentativa é contada mesmo se falhar. **Nunca retry**.

## Saída

Prévia e sucesso: `numero`, `id_documento`, `processo` (`nup`, `id`), `unidade`, `motivo`, `origem:"web"`, `confirmado`, `alterado`, `requisicoes`, `reconciliado_por_leitura`. Em falha: `codigo`, `subcodigo`, `erro`, `retentavel:false`, `status_http` e mensagem saneada quando observados; em erro depois do POST, `falha_do_envio` e `verificacao`. Nenhum cookie, URL assinada ou `infra_hash` aparece.

## Subcódigos específicos

- `barra_do_cancelamento_fora_da_forma`
- `links_de_cancelamento_repetidos`
- `endereco_de_cancelamento_nao_reconhecido`
- `endereco_de_cancelamento_de_outro_documento`
- `cancelamento_nao_oferecido_na_barra`
- `documento_ja_cancelado`
- `estado_antes_do_cancelamento_nao_observado`
- `tela_do_cancelamento_redirecionada`
- `charset_da_tela_de_cancelamento_nao_suportado`
- `formulario_de_cancelamento_de_outro_documento`
- `campo_de_motivo_do_cancelamento_ausente`
- `redirecionamento_do_cancelamento_nao_reconhecido`
- `resposta_do_cancelamento_sem_redirecionamento`
- `cancelamento_nao_aplicado`
- `efeito_incerto_no_cancelamento`

A tela de login, o transporte e o primitivo de formulário propagam seus próprios subcódigos. Entrada inválida: `motivo_vazio`, `motivo_com_controle`, `motivo_fora_da_codificacao` e os da validação de DOC/NUP.

## Evidências e lacunas

- Observações de referência: Todos @323eb01 `src/todos/cancelamento.py` e `tests/test_documento_mover_cancelar_010.py`; acervo de GO (SEI 5.0, 2026-10-19), UFRJ 4.1.5 e PEN3. UnB 4.0.9 não publicou a ação com o usuário de treinamento observado.
- Testes sintéticos da barra em `src/documento_cancelar.rs` e sequências de requisição em `src/documento_cancelar/tests_fluxo.rs`.
- Replay do acervo: **NOT_RUN**. Ensaio live prévia/POST/verificação em treinamento: **NOT_RUN**, 0 requisições reais; nenhum resíduo no SEI.
