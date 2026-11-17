---
type: ImplementationDecision
title: Pré-inspeção local de PDF sem habilitar a juntada remota
operation: documento.incluir-externo
implementation: partial
status: adopted-for-scope
scope: Verificação offline do arquivo e bloqueio de confirmação até cadastro real observado
selected_strategy: pdf-local-antes-de-formulario-publicado
alternatives: pdf-local-antes-de-formulario-publicado,upload-direto,upload-sem-conferir
evidence_level: todos-unb-2026-10-11-e-teste-sintetico
live_validation: not-run
---

# Decisão — documento externo PDF (#332)

## Contexto observado

O Todos documentou em 11/10/2026, em UnB de treinamento, a seleção do documento externo em `documento_escolher_tipo`, o cadastro `documento_receber`, o upload temporário multipart `documento_upload_anexo` e a referência `hdnAnexos` no cadastro definitivo. O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) mostra que a árvore pode não exibir o anexo embora ele esteja disponível na alteração do cadastro. Esses cenários não demonstram, por si, o funcionamento do IES em outras instalações.

## Alternativas consideradas

1. **Inspecionar primeiro apenas o arquivo local** (escolhida nesta fatia): não iniciar sessão nem rede até confirmar as pré-condições locais; não afirmar que tipo, tamanho ou upload remoto foram aceitos.
2. Fazer diretamente upload/cadastro sem ler a tela: rejeitada, pois exigiria inventar URLs, campos e resultado.
3. Aceitar recibo de upload como prova de juntada: rejeitada, porque upload temporário e inclusão definitiva são efeitos distintos.

## Comparação

| Critério | Inspeção local primeiro | Upload direto | Recibo como prova |
| --- | --- | --- | --- |
| Pré-condições locais sem rede | Sim | Não | Não |
| Origem do formulário autorizada | Ainda pendente | Inventada | Desconhecida |
| Dois efeitos mutadores distinguíveis | Na próxima fatia | Não | Não |
| Efeito remoto declarado como comprovado | Não | Arriscado | Indevido |

## Decisão


A inspeção identifica arquivo regular, nome sem `&#` (restrição observada pelo Todos) nem controles de terminal, extensão PDF, início `%PDF-`, tamanho e SHA-256 em fluxo, sem carregar o documento todo. O prefixo não é validação completa do formato. Uma releitura dos metadados pelo mesmo descritor após o hash recusa alteração detectável durante a inspeção, inclusive quando o tamanho final é idêntico. Isso não torna atômica a resolução de ancestrais e não garante que o arquivo permaneça igual até futuro upload remoto; a implementação do upload deverá vincular os bytes efetivamente enviados à inspeção. A operação rejeita links simbólicos e limites **somente quando recebidos da tela**, sem limite arbitrário. Falhas permanecem no enum tipado `FalhaPdf` e são serializadas pelo próprio erro com Serde, sem subcódigo paralelo; o comando continua sem mutação de qualquer serviço e sem acesso ao SEI.

O comando `--confirmar` é explicitamente bloqueado antes de ler a sessão. Não há dois POSTs implementados: na próxima fatia, o upload temporário e o cadastro terão efeitos observados e erros distintos, com releitura independente, saída 1 se houver dúvida e nunca retry. Isso é intencional, não uma capacidade declarada como completa.

## Evidências verificáveis

- [Contrato parcial](../../contracts/cli/documento-incluir-externo.md).
- [Evidência da operação derivada](../../contracts/evidencias/documento-incluir-externo.md).
- [Testes da CLI offline](../../tests/cli_documento_incluir_externo.rs).
- [Evidência de RED e ausência de escrita](../validacao-live/2026-11-06-documento-incluir-externo-local.md).

## Limitações e revisão


RED dos testes locais no CI e GREEN sujeitos aos gates do head exato da [PR #352](https://github.com/franklinbaldo/ies/pull/352). No ensaio vivo desta fatia: `NOT_RUN`, 0 GET, 0 POST, 0 requisições e 0 resíduos.

Contrato: [documento incluir externo](../../contracts/cli/documento-incluir-externo.md).
