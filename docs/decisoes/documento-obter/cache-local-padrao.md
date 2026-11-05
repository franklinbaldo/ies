---
type: ImplementationDecision
title: Cache local padrão dos documentos lidos
operation: documento.obter
implementation: pending
status: adopted-for-scope
scope: cache automático por instalação e número SEI, independente do processo; saída opcional a partir do cache; sem requisições adicionais
selected_strategy: cache-por-conteudo-e-versao
alternatives: cache-por-conteudo-e-versao,cache-por-processo,apenas-saida-explicita,sobrescrever-foto
evidence_level: decisao-do-proprietario-e-inspecao-do-codigo
live_validation: not-run
---

# Cache local padrão de documento (#236)

Decisão do proprietário de 05/11/2026, com complemento nos comentários da
[issue #236](https://github.com/franklinbaldo/ies/issues/236).
Complementa [documento.obter](../documento-obter.md) e substitui, para
o armazenamento interno, a dependência de `--saida` da
[foto idempotente](saida-idempotente.md).

## Alternativas e escolha

- **Cache por conteúdo e versão (adotado):** a leitura guarda
  `${XDG_CACHE_HOME:-$HOME/.cache}/ies/<instalacao>/<documento>/v<N>-<sha256[:16]>.<ext>`.
  Sem versão observada, usa `x-<sha256[:16]>.<ext>`. Não coloca o
  processo no caminho, porque documentos podem mudar de processo.
- **Cache por processo:** descartado porque `DOCUMENTO_MOVIDO` deixaria
  duplicatas e confundiria a identidade durável; o índice local conserva
  a relação documento↔processo.
- **Apenas `--saida`:** é o comportamento anterior; uma leitura sem
  a opção perde os bytes do documento.
- **Sobrescrever foto:** descartado porque poderia apagar edição local e
  ocultaria diferenças na versão servida.

## Contrato de segurança e semântica

A escrita automática deve usar
`arquivo_privado::gravar(..., Politica::exclusiva().diretorio_do_ies().criando_diretorios())`.
Os diretórios criados ficam `0700`, os arquivos `0600`, com
`hard_link` exclusivo para publicar e sem seguir symlink final.
`DestinoJaExiste` só será no-op depois de verificar que o arquivo
existente é regular, privado e tem o SHA-256 integral esperado; a
coincidência dos 16 primeiros hex no nome **não** dispensa verificação.
Nunca escrever HTML contendo link assinado, `infra_hash` ou cookie.

O host da instalação, previamente validado pela infraestrutura de
`Instalacao`, é o único discriminador do primeiro nível. O documento
é o número SEI, não o identificador do processo. A extensão do externo
vem do MIME permitido e não do nome recebido do servidor.

`--json` acrescenta `cache: {caminho, versao, sha256}` após a
gravação, ou `null` se o cache foi desativado. Acrescenta também
`crc_rodape`, apenas informativo: não é identidade, verificação
criptográfica ou pré-condição da gravação; quando não observado é
`null`. O IES não recalcula o CRC do SEI.

`--sem-cache` ou `IES_CACHE_DOCUMENTOS=0` desativam a escrita do
cache. `--saida DIR` mantém a cópia explícita `DOC.vN.html` para
internos, preferindo `hard_link` a partir do arquivo em cache, com
fallback a cópia e mantendo `foto_local_divergente`. O cache não pode
exigir requisições adicionais: versão desconhecida permanece
desconhecida, sem consultar a lista apenas para nomear o arquivo.

## Implicações para a implementação

O leitor atual só produz `versoes.foto` quando solicita uma foto por
`--saida`; o fluxo de externos cria arquivo parcial somente com
destino explícito. Ambos precisam preservar os bytes aptos para
cache mesmo sem `--saida`, respeitando os limites do anexo e sem
expor conteúdo ou URLs à saída padrão. Uma leitura pode produzir
`cache` e `arquivo` (a cópia em `--saida`) como campos distintos.

A idempotência da cópia explícita compara o conteúdo com a foto local
antes da gravação. O cache por hash guarda múltiplas versões ou bytes
distintos, sem sobrescrever; esta diferença de semântica é intencional.

## Evidência, limites e revisão

A decisão foi definida pelo proprietário; houve inspeção do código
de leitura, de `arquivo_privado`, `foto_documento` e dos contratos.
**Esta primeira alteração documental não implementa a decisão.**
RED, GREEN, gate, CI e ensaio live permanecem não executados nesta
etapa. Não atribuir PASS ao ensaio sem execução. O código e os testes
devem ser concluídos antes de marcar a PR pronta.
