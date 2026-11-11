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
- `--motivo TEXTO` é obrigatório e segue sem alteração para o formulário, incluindo espaços e quebras de linha. O charset da tela determina a codificação; um valor não representável é recusado pelo codificador antes do POST. A CLI não impõe `trim`, conteúdo mínimo ou Latin-1 fixo.
- `--confirmar` opcional. Sem ele há apenas prévia; com ele executa **um POST** e jamais retry.
- `--json` e `--unidade` seguem as opções globais.

## Ação e formulário

1. Abre o documento até a árvore, confere NUP, id e número SEI. Exige `natureza.cancelado == false`. Se já cancelado, recusa sem abrir a tela. Cancelamento com uma só marca já é recusado pela leitura da árvore (`ErroArvore`), antes de qualquer escrita.
2. Da única barra `Nos[i].acoes` **desse documento** obtém a única âncora cujo href traz `acao=documento_cancelar`; ausência recusa `ErroCancelar::CancelamentoNaoOferecido`. Confere o endereço relativo contra a instalação, `id_procedimento`, `id_documento` e `infra_hash` único (64 hex). Duplicata, outro documento ou URL suspeita são recusados.
3. Faz GET da tela publicada; 302 não é seguido. Confere `form#frmDocumentoCancelar`, `method=post`, `action` do mesmo processo e documento, um `textarea name=txaMotivo` habilitado, o oculto `hdnInfraTipoPagina` e único `button type=submit name=sbmSalvar`. Não aceita campos enviáveis adicionais.
4. Deriva a submissão pelo primitivo compartilhado `formulario_web`, com `EfeitoAvaliado::AlteraEstadoDoProcesso { CancelamentoDoDocumento }`, `txaMotivo` e o botão `sbmSalvar`; `charset` da tela.
5. **Prévia** publica identidade, motivo, `confirmado:false`, `alterado:false` e contagem, sem POST. **Confirmação** faz um único POST. A resposta esperada é 302 sem seguir para `arvore_visualizar` do mesmo processo/documento, com `atualizar_arvore=1`. Mesmo 5xx, login, rede ou 302 inesperado **não** autorizam retry.
6. Sempre uma nova leitura da árvore após qualquer envio possivelmente feito; só considera sucesso o ícone `documento_cancelado` juntamente da dica `Documento Cancelado: <motivo>` igual ao solicitado. Se não houve cancelamento demonstrado, saída 1; leitura falha ou efeito ambíguo, saída 1.

## Contagem de requisições

Abertura do documento, pelo `--processo` a frio: 3 GET. Prévia: +1 GET da tela (4). `--confirmar`: +1 POST e +1 GET para verificação (6). Caminhos aquecidos e pesquisa rápida têm custos próprios da abertura; cada tentativa é contada mesmo se falhar. **Nunca retry**.

## Saída

Sem unidade publicada (nem pela árvore nem por `--unidade`), a escrita não
é montada: falha com `UnidadeNaoPublicada`, saída 1, nada enviado.
Prévia e sucesso: `numero`, `id_documento`, `processo` (`nup`, `id`), `unidade` (sempre texto), `motivo`, `origem:"web"`, `confirmado`, `alterado`, `requisicoes`, `reconciliado_por_leitura`. Em falha: saída 1, stdout vazio e `Error: <ErroCancelar::…>` em stderr; o status HTTP e a mensagem saneada vêm em `Transporte(Status(..))` quando observados, e depois do POST os erros carregam `falha_do_envio` e `verificacao`, que é `Lida`, `LeituraFalhou` ou `DocumentoDiferenteNaReleitura` (a releitura abriu outro documento; a escrita não é atribuída ao pedido). Nenhum cookie, URL assinada ou `infra_hash` aparece.

## Subcódigos específicos

| Erro (stderr) | Fato |
| --- | --- |
| `ErroCancelar::UnidadeNaoPublicada` | Nenhuma unidade publicada para a escrita; nada é enviado |
| `ErroLinkCancelar::BarraForaDaForma` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroLinkCancelar::LinksRepetidos` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroLinkCancelar::EnderecoNaoReconhecido` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroLinkCancelar::EnderecoDeOutroDocumento` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::CancelamentoNaoOferecido` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `DocumentoJaCancelado` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::TelaRedirecionada` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::CharsetNaoSuportado` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::FormularioDeOutroDocumento` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::MotivoNaoPublicadoNaTela` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `FalhaDoCancelamento::RedirecionamentoNaoReconhecido` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `FalhaDoCancelamento::RespostaSemRedirecionamento` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `CancelamentoNaoAplicado` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |
| `ErroCancelar::NaoConfirmado` | Recusa tipada na leitura, validação, envio ou verificação conforme o estágio |

A tela de login, o transporte, o primitivo de formulário e a validação de DOC/NUP mantêm os subcódigos desse contrato.

### Motivo como texto opaco

O pedido recebe `&str`. A ausência do argumento é erro de uso; o conteúdo fornecido não cria um erro local de motivo. A representabilidade usa a falha tipada do formulário para o campo `txaMotivo`, conforme a codificação da tela. Prévia e resultado preservam o texto solicitado; a reconciliação continua exigindo o motivo publicado pela árvore.

## Evidências e lacunas

- Observações de referência: Todos @323eb01 `src/todos/cancelamento.py` e `tests/test_documento_mover_cancelar_010.py`; acervo de GO (SEI 5.0, 2026-10-19), UFRJ 4.1.5 e PEN3. UnB 4.0.9 não publicou a ação com o usuário de treinamento observado.
- Testes sintéticos da barra em `src/documento_cancelar.rs` e sequências de requisição em `src/documento_cancelar/tests_fluxo.rs`.
- Replay do acervo: **NOT_RUN**. Ensaio live prévia/POST/verificação em treinamento: **NOT_RUN**, 0 requisições reais; nenhum resíduo no SEI.
