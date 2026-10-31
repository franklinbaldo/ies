---
type: CommandContract
title: Prévia e salvamento seguro de documento
operation: documento.salvar
command: ies documento salvar DOC
implementation: partial
effect: write
---

# Prévia local e salvamento de documento

`ies documento salvar DOC [--versao N] [--confirmar]` consulta a cópia
local versionada no diretório atual. Sem `--versao`, seleciona a maior
versão **local** observada; isso não afirma a versão remota atual.

Uma cópia segura exige `DOC.vN.html`, `.DOC.vN.base.html` e
`.DOC.vN.meta.json`, com identidade de instalação, ID do documento,
versão positiva e hash SHA-256 da base. O arquivo da visualização
histórica (`DOC/versao-N.html`) nunca é uma cópia de trabalho.

A prévia é **offline**, compara a cópia com a base, não expõe o corpo,
URLs assinadas ou cookies, e não abre o editor. Falhas de base,
identidade, links simbólicos ou inconsistências têm subcódigos tipados.

`--confirmar` **não envia POST no SEI**: retorna
`escrita_remota_nao_validada` após conferir a cópia. A escrita Web,
mesclagem por seção, concorrência, assinatura e promoção para uma nova
versão dependem de validação específica por instalação; não podem
ser simuladas. O ciclo completo de #89 permanece pendente.

- [Evidências legadas](../evidencias/documento-salvar.md)
- [Decisão arquitetural](../../docs/decisoes/documento-salvar.md)
- [Testes de CLI](../../tests/cli_contract.rs)

## Localização conservadora das alterações (Refs #89)

A prévia local também publica `trecho_divergente`: quando o HTML de
trabalho difere da base, informa os intervalos de linhas **zero-based,
fim exclusivo** na base e no trabalho que contêm todas as alterações.
Compara os prefixos e sufixos idênticos de linhas, sem precisar abrir
a sessão SEI, sem imprimir conteúdo e com memória constante.
Duas ou mais edições separadas podem produzir um único trecho contendo
linhas inalteradas entre elas: **não** equivale a um diff mínimo nem
prova equivalência com versão remota. Sem alteração, o campo é `null`.
A comparação é por bytes separados por LF, inclusive com HTML não UTF-8,
sem alterar o conteúdo de qualquer arquivo.

A inspeção permanece offline, sempre submetida à conferência de
metadados, identidade, hash da base e à recusa de arquivos simbólicos.
Nenhuma opção de `salvar --confirmar` passa a executar escrita remota.


## Leitura privada dos arquivos locais (Refs #89)

A prévia lê trabalho, base e manifesto como bytes, nunca como conteúdo
remoto. Em Linux, cada arquivo deve ser aberto com proteção contra
link simbólico no componente final e contra bloqueio em arquivo especial;
a segurança (arquivo regular, limite 8 MB e permissão 0600/sem acesso
a grupo ou terceiros) é conferida no **descritor já aberto**, e não apenas
com `symlink_metadata` prévio. Um link simbólico retorna
`copia_local_link_simbolico` e permissão excessiva retorna
`copia_local_permissoes_inseguras`, ambos erros tipados antes
de publicar qualquer prévia. Dados não UTF-8 permanecem bytes opacos.
Não emitir o conteúdo de cópias na saída de diagnóstico.

Os controles de Linux reduzem a janela de troca do componente final,
mas **não** constituem defesa universal contra substituição de
diretórios ancestrais ou alteração simultânea dos bytes após leitura.
Não há operação Web, escrita no SEI ou afirmação de versão remota.


## Inventário seguro de cópias locais (Refs #89)

`ies documento salvar DOC --listar-copias [--json]` enumera **somente**
cópias de trabalho `DOC.vN.html` observadas no diretório atual, em
ordem **numérica** de versão, sem acesso à rede. Não recebe `--versao`
nem `--confirmar`; não equivale a listar versões existentes no SEI.

Cada entrada precisa ter base `.DOC.vN.base.html` e manifesto
`.DOC.vN.meta.json` válidos para o mesmo documento, ID interno,
instalação e hash SHA-256 da base, seguindo exatamente as invariantes
da prévia. Não descartar cópias corrompidas para retornar sucesso
parcial; recusar a lista inteira com subcódigo tipado. Rejeitar
diretório simbólico, versão inválida, duas grafias para a mesma
versão e coleções além do limite local estabelecido.

A saída JSON contém `copias` (projeção de `PreviaCopia`),
`escopo: copias_de_trabalho_locais_verificadas`,
`versao_remota_confirmada: false` e `escrita_remota: false`.
Uma lista vazia significa apenas ausência de cópias para esse número
no diretório observado. Nunca publicar conteúdo HTML nem alegar
versão atual no servidor. Sem comandos Web adicionais.
